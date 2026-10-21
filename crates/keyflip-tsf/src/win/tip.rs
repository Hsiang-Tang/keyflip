//! The text service itself: key sinks, focus tracking, composition.
//!
//! Everything that decides anything lives in the engine server. This file
//! translates keys into requests and turns the answer into TSF calls.

use std::cell::{Cell, RefCell};
use std::mem::ManuallyDrop;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use keyflip_proto::{Request, Response};
use windows::core::{implement, Interface, Result, GUID, VARIANT};
use windows::Win32::Foundation::{BOOL, E_FAIL, E_INVALIDARG, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardState, ToUnicodeEx,
};
use windows::Win32::UI::TextServices::*;
use windows::Win32::UI::WindowsAndMessaging::{GetGUIThreadInfo, GUITHREADINFO};
use windows::Win32::Graphics::Gdi::ClientToScreen;

use super::candwin::CandWindow;
use super::dattr::{input_guid, selected_guid, DaEnum, DaInfo};
use super::Live;
use crate::keymap::{self, Mods};
use crate::pipe::Link;
use crate::render::{RunKind, View};

/// State shared by the text service and its edit sessions. Holds nothing the
/// engine decides: only COM handles, the connection and cached view data.
pub struct Shared {
    tid: Cell<u32>,
    link: RefCell<Link>,
    comp: RefCell<Option<ITfComposition>>,
    comp_ctx: RefCell<Option<ITfContext>>,
    cand: RefCell<Option<CandWindow>>,
    atoms: Cell<(u32, u32)>,
    /// What is currently shown, to skip edit sessions that would change nothing.
    last: RefCell<View>,
    /// (context pointer, is password-like) for the focused context.
    scope: Cell<Option<(usize, bool)>>,
    /// Virtual key whose key-down we consumed, so its key-up is consumed too.
    eaten: Cell<u32>,
}

impl Shared {
    fn new() -> Shared {
        Shared {
            tid: Cell::new(0),
            link: RefCell::new(Link::new(super::module())),
            comp: RefCell::new(None),
            comp_ctx: RefCell::new(None),
            cand: RefCell::new(None),
            atoms: Cell::new((0, 0)),
            last: RefCell::new(View::default()),
            scope: Cell::new(None),
            eaten: Cell::new(0),
        }
    }

    fn hide_candidates(&self) {
        if let Some(w) = self.cand.borrow().as_ref() {
            w.hide();
        }
    }

    fn send_reset(&self) {
        if let Ok(mut l) = self.link.try_borrow_mut() {
            let _ = l.call_if_connected(&Request::Reset);
        }
    }
}

// ---------------------------------------------------------------- sessions

/// Run an edit session: synchronously when the application allows it,
/// otherwise asynchronously. Returns whether the request was accepted.
fn request(sh: &Shared, ctx: &ITfContext, es: &ITfEditSession, write: bool, allow_sync: bool) -> bool {
    let base = if write { TF_ES_READWRITE.0 } else { TF_ES_READ.0 };
    let tid = sh.tid.get();
    unsafe {
        if allow_sync {
            // Err means the request itself was refused (for example another
            // session holds the lock); a failing session comes back as Ok(hr).
            if let Ok(hr) = ctx.RequestEditSession(
                tid,
                es,
                TF_CONTEXT_EDIT_CONTEXT_FLAGS(base | TF_ES_SYNC.0),
            ) {
                return hr.is_ok();
            }
        }
        matches!(
            ctx.RequestEditSession(tid, es, TF_CONTEXT_EDIT_CONTEXT_FLAGS(base | TF_ES_ASYNC.0)),
            Ok(hr) if hr.is_ok()
        )
    }
}

unsafe fn selection_range(ctx: &ITfContext, ec: u32) -> Result<ITfRange> {
    let mut sel = [TF_SELECTION::default()];
    let mut n = 0u32;
    ctx.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut n)?;
    if n == 0 {
        return Err(E_FAIL.into());
    }
    ManuallyDrop::take(&mut sel[0].range).ok_or_else(|| E_FAIL.into())
}

unsafe fn set_selection(ctx: &ITfContext, ec: u32, range: &ITfRange) -> Result<()> {
    let mut sel = TF_SELECTION {
        range: ManuallyDrop::new(Some(range.clone())),
        style: TF_SELECTIONSTYLE {
            ase: TF_AE_NONE,
            fInterimChar: BOOL(0),
        },
    };
    let r = ctx.SetSelection(ec, std::slice::from_ref(&sel));
    ManuallyDrop::drop(&mut sel.range);
    r
}

unsafe fn clear_attributes(ctx: &ITfContext, ec: u32, range: &ITfRange) {
    if let Ok(prop) = ctx.GetProperty(&GUID_PROP_ATTRIBUTE) {
        let _ = prop.Clear(ec, range);
    }
}

/// Screen rectangle of the composition (or of the caret when there is none).
unsafe fn anchor_rect(sh: &Shared, ctx: &ITfContext, ec: u32) -> Option<RECT> {
    let comp = sh.comp.borrow().clone();
    let range = match comp {
        Some(c) => c.GetRange().ok(),
        None => selection_range(ctx, ec).ok(),
    };
    if let (Some(range), Ok(view)) = (range, ctx.GetActiveView()) {
        let mut rc = RECT::default();
        let mut clipped = BOOL(0);
        if view.GetTextExt(ec, &range, &mut rc, &mut clipped).is_ok() && plausible(&rc) {
            return Some(rc);
        }
    }
    caret_rect()
}

/// A text extent that is not (0,0) and not empty is believable.
fn plausible(rc: &RECT) -> bool {
    !(rc.left == 0 && rc.top == 0 && rc.right == 0 && rc.bottom == 0)
        && (rc.bottom > rc.top)
        && !(rc.left == 0 && rc.top == 0)
}

/// Caret of the foreground thread's focus window, in screen coordinates.
fn caret_rect() -> Option<RECT> {
    unsafe {
        let mut gti = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        GetGUIThreadInfo(0, &mut gti).ok()?;
        let hwnd = gti.hwndCaret;
        if hwnd.0.is_null() {
            return None;
        }
        let rc = gti.rcCaret;
        if rc.bottom <= rc.top {
            return None;
        }
        let mut tl = POINT { x: rc.left, y: rc.top };
        let mut br = POINT { x: rc.right, y: rc.bottom };
        if !ClientToScreen(hwnd, &mut tl).as_bool() || !ClientToScreen(hwnd, &mut br).as_bool() {
            return None;
        }
        let out = RECT {
            left: tl.x,
            top: tl.y,
            right: br.x,
            bottom: br.y,
        };
        plausible(&out).then_some(out)
    }
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Commit text and/or update the composition, then place the candidate window.
unsafe fn apply(
    sh: &Rc<Shared>,
    ctx: &ITfContext,
    ec: u32,
    commit: &str,
    view: &View,
) -> Result<()> {
    if !commit.is_empty() {
        let text = utf16(commit);
        let existing = sh.comp.borrow_mut().take();
        sh.comp_ctx.borrow_mut().take();
        if let Some(comp) = existing {
            let range = comp.GetRange()?;
            range.SetText(ec, 0, &text)?;
            clear_attributes(ctx, ec, &range);
            // Collapse to the end of what was written, or the next character
            // typed would replace it (Notepad).
            range.Collapse(ec, TF_ANCHOR_END)?;
            set_selection(ctx, ec, &range)?;
            comp.EndComposition(ec)?;
        } else {
            let ins: ITfInsertAtSelection = ctx.cast()?;
            let range = ins.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &text)?;
            range.Collapse(ec, TF_ANCHOR_END)?;
            set_selection(ctx, ec, &range)?;
        }
    }

    if view.text.is_empty() {
        let existing = sh.comp.borrow_mut().take();
        sh.comp_ctx.borrow_mut().take();
        if let Some(comp) = existing {
            let range = comp.GetRange()?;
            range.SetText(ec, 0, &[])?;
            comp.EndComposition(ec)?;
        }
    } else {
        let existing = sh.comp.borrow().clone();
        let comp = match existing {
            Some(c) => c,
            None => {
                let ins: ITfInsertAtSelection = ctx.cast()?;
                let at = ins.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])?;
                let cc: ITfContextComposition = ctx.cast()?;
                let sink: ITfCompositionSink = CompSink {
                    sh: sh.clone(),
                    _live: Live::new(),
                }
                .into();
                let c = cc.StartComposition(ec, &at, &sink)?;
                *sh.comp.borrow_mut() = Some(c.clone());
                *sh.comp_ctx.borrow_mut() = Some(ctx.clone());
                c
            }
        };
        let range = comp.GetRange()?;
        range.SetText(ec, 0, &view.text)?;
        mark_runs(sh, ctx, ec, &range, view);
        // Caret at the engine's cursor offset.
        if let Ok(caret) = range.Clone() {
            let mut moved = 0i32;
            if caret.Collapse(ec, TF_ANCHOR_START).is_ok()
                && caret
                    .ShiftEnd(ec, view.cursor as i32, &mut moved, std::ptr::null())
                    .is_ok()
                && caret.Collapse(ec, TF_ANCHOR_END).is_ok()
            {
                let _ = set_selection(ctx, ec, &caret);
            }
        }
    }

    match &view.cand {
        None => sh.hide_candidates(),
        Some(cv) => match anchor_rect(sh, ctx, ec) {
            None => sh.hide_candidates(),
            Some(rc) => {
                if sh.cand.borrow().is_none() {
                    *sh.cand.borrow_mut() = CandWindow::new();
                }
                match sh.cand.borrow().as_ref() {
                    Some(w) => w.show(cv, rc),
                    None => {}
                }
            }
        },
    }
    *sh.last.borrow_mut() = view.clone();
    Ok(())
}

unsafe fn mark_runs(sh: &Shared, ctx: &ITfContext, ec: u32, whole: &ITfRange, view: &View) {
    let Ok(prop) = ctx.GetProperty(&GUID_PROP_ATTRIBUTE) else {
        return;
    };
    let (input, selected) = sh.atoms.get();
    for run in &view.runs {
        let Ok(r) = whole.Clone() else { continue };
        let mut moved = 0i32;
        let none = std::ptr::null();
        let ok = r.Collapse(ec, TF_ANCHOR_START).is_ok()
            && r.ShiftEnd(ec, (run.start + run.len) as i32, &mut moved, none).is_ok()
            && r.ShiftStart(ec, run.start as i32, &mut moved, none).is_ok();
        if !ok {
            continue;
        }
        let atom = match run.kind {
            RunKind::Input => input,
            RunKind::Selected => selected,
        };
        let _ = prop.SetValue(ec, &r, &VARIANT::from(atom as i32));
    }
}

#[implement(ITfEditSession)]
struct ApplySession {
    sh: Rc<Shared>,
    ctx: ITfContext,
    commit: String,
    view: View,
    _live: Live,
}

impl ITfEditSession_Impl for ApplySession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| unsafe {
            apply(&self.sh, &self.ctx, ec, &self.commit, &self.view)
        }))
        .unwrap_or_else(|_| Err(E_FAIL.into()))
    }
}

/// Throw the composition away without committing anything.
#[implement(ITfEditSession)]
struct CancelSession {
    comp: ITfComposition,
    _live: Live,
}

impl ITfEditSession_Impl for CancelSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| unsafe {
            let range = self.comp.GetRange()?;
            range.SetText(ec, 0, &[])?;
            self.comp.EndComposition(ec)
        }))
        .unwrap_or_else(|_| Err(E_FAIL.into()))
    }
}

/// Reads the context's input scope: password-like fields get no keys.
#[implement(ITfEditSession)]
struct ScopeSession {
    sh: Rc<Shared>,
    ctx: ITfContext,
    key: usize,
    _live: Live,
}

unsafe fn is_password_scope(ctx: &ITfContext, ec: u32) -> bool {
    let Ok(prop) = ctx.GetProperty(&GUID_PROP_INPUTSCOPE) else {
        return false;
    };
    let range = match selection_range(ctx, ec) {
        Ok(r) => r,
        Err(_) => match ctx.GetStart(ec) {
            Ok(r) => r,
            Err(_) => return false,
        },
    };
    let Ok(var) = prop.GetValue(ec, &range) else {
        return false;
    };
    let Ok(unk) = windows::core::IUnknown::try_from(&var) else {
        return false;
    };
    let Ok(scope) = unk.cast::<ITfInputScope>() else {
        return false;
    };
    let mut list: *mut InputScope = std::ptr::null_mut();
    let mut n = 0u32;
    if scope.GetInputScopes(&mut list, &mut n).is_err() || list.is_null() {
        return false;
    }
    let found = std::slice::from_raw_parts(list, n as usize)
        .iter()
        .any(|s| *s == IS_PASSWORD || *s == IS_NUMERIC_PASSWORD || *s == IS_NUMERIC_PIN);
    CoTaskMemFree(Some(list.cast()));
    found
}

impl ITfEditSession_Impl for ScopeSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let secure = catch_unwind(AssertUnwindSafe(|| unsafe { is_password_scope(&self.ctx, ec) }))
            .unwrap_or(true);
        self.sh.scope.set(Some((self.key, secure)));
        Ok(())
    }
}

#[implement(ITfCompositionSink)]
struct CompSink {
    sh: Rc<Shared>,
    _live: Live,
}

impl ITfCompositionSink_Impl for CompSink_Impl {
    /// The application ended our composition (clicked elsewhere, for example).
    fn OnCompositionTerminated(&self, _ec: u32, _comp: Option<&ITfComposition>) -> Result<()> {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            self.sh.comp.borrow_mut().take();
            self.sh.comp_ctx.borrow_mut().take();
            *self.sh.last.borrow_mut() = View::default();
            self.sh.hide_candidates();
            self.sh.send_reset();
        }));
        Ok(())
    }
}

/// End any composition without committing text, and forget it.
fn cancel_composition(sh: &Rc<Shared>) {
    let comp = sh.comp.borrow_mut().take();
    let ctx = sh.comp_ctx.borrow_mut().take();
    *sh.last.borrow_mut() = View::default();
    sh.hide_candidates();
    if let (Some(comp), Some(ctx)) = (comp, ctx) {
        let es: ITfEditSession = CancelSession {
            comp,
            _live: Live::new(),
        }
        .into();
        // Focus callbacks do not allow synchronous sessions.
        let _ = request(sh, &ctx, &es, true, false);
    }
}

// ----------------------------------------------------------- text service

#[implement(
    ITfTextInputProcessorEx,
    ITfTextInputProcessor,
    ITfKeyEventSink,
    ITfThreadMgrEventSink,
    ITfDisplayAttributeProvider
)]
pub struct TextService {
    sh: Rc<Shared>,
    tm: RefCell<Option<ITfThreadMgr>>,
    cookie: Cell<u32>,
    _live: Live,
}

impl TextService {
    pub fn new() -> TextService {
        TextService {
            sh: Rc::new(Shared::new()),
            tm: RefCell::new(None),
            cookie: Cell::new(0),
            _live: Live::new(),
        }
    }

    fn activate(&self, tm: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        let tm = tm.ok_or_else(|| windows::core::Error::from(E_INVALIDARG))?.clone();
        self.sh.tid.set(tid);
        unsafe {
            let cats: ITfCategoryMgr =
                CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
            self.sh
                .atoms
                .set((cats.RegisterGUID(&input_guid())?, cats.RegisterGUID(&selected_guid())?));

            let keys: ITfKeystrokeMgr = tm.cast()?;
            let sink: ITfKeyEventSink = self.cast()?;
            keys.AdviseKeyEventSink(tid, &sink, true)?;

            let src: ITfSource = tm.cast()?;
            let tsink: windows::core::IUnknown = self.cast::<ITfThreadMgrEventSink>()?.cast()?;
            match src.AdviseSink(&ITfThreadMgrEventSink::IID, &tsink) {
                Ok(c) => self.cookie.set(c),
                Err(e) => {
                    let _ = keys.UnadviseKeyEventSink(tid);
                    return Err(e);
                }
            }
        }
        *self.tm.borrow_mut() = Some(tm);
        Ok(())
    }

    fn deactivate(&self) {
        cancel_composition(&self.sh);
        self.sh.send_reset();
        if let Some(tm) = self.tm.borrow_mut().take() {
            unsafe {
                if self.cookie.get() != 0 {
                    if let Ok(src) = tm.cast::<ITfSource>() {
                        let _ = src.UnadviseSink(self.cookie.get());
                    }
                    self.cookie.set(0);
                }
                if let Ok(keys) = tm.cast::<ITfKeystrokeMgr>() {
                    let _ = keys.UnadviseKeyEventSink(self.sh.tid.get());
                }
            }
        }
        self.sh.cand.borrow_mut().take();
        self.sh.scope.set(None);
        self.sh.tid.set(0);
    }

    /// Translate a key-down into a protocol key, or `None` to pass it through.
    fn key_of(&self, wparam: WPARAM, lparam: LPARAM) -> Option<keyflip_proto::Key> {
        let vk = wparam.0 as u32;
        let scan = ((lparam.0 >> 16) & 0xFF) as u32;
        let extended = (lparam.0 >> 24) & 1 == 1;
        // Cheap rejection first: no keyboard state work for keys we never send.
        keymap::code_of(vk, scan, extended)?;
        let mut state = [0u8; 256];
        unsafe { GetKeyboardState(&mut state).ok()? };
        let mods: Mods = keymap::mods_from_state(&state);
        let mut buf = [0u16; 8];
        let n = unsafe {
            // Flag 0x4: do not change the keyboard's dead-key state.
            ToUnicodeEx(vk, scan, &state, &mut buf, 0x4, GetKeyboardLayout(0))
        };
        let text = if n > 0 {
            keymap::text_from_utf16(&buf[..n as usize])
        } else {
            None
        };
        keymap::translate(vk, scan, extended, mods, text)
    }

    /// Password-like or read-only contexts get no keys and send nothing.
    fn blocked(&self, pic: &ITfContext) -> bool {
        unsafe {
            if let Ok(st) = pic.GetStatus() {
                if st.dwDynamicFlags & TS_SD_READONLY != 0 || st.dwStaticFlags & TS_SS_TRANSITORY != 0 {
                    return true;
                }
            }
        }
        let key = pic.as_raw() as usize;
        if let Some((k, secure)) = self.sh.scope.get() {
            if k == key {
                return secure;
            }
        }
        // Unknown context: blocked until the input scope has been read.
        self.sh.scope.set(Some((key, true)));
        let es: ITfEditSession = ScopeSession {
            sh: self.sh.clone(),
            ctx: pic.clone(),
            key,
            _live: Live::new(),
        }
        .into();
        request(&self.sh, pic, &es, false, true);
        self.sh.scope.get().map_or(true, |(_, s)| s)
    }

    fn test_key(&self, pic: Option<&ITfContext>, wparam: WPARAM, lparam: LPARAM) -> bool {
        let Some(pic) = pic else { return false };
        let Some(key) = self.key_of(wparam, lparam) else {
            return false;
        };
        if self.blocked(pic) {
            return false;
        }
        match self.sh.link.borrow_mut().call(&Request::Test { key }) {
            Some(Response::Test { handled }) => handled,
            _ => false,
        }
    }

    fn do_key(&self, pic: Option<&ITfContext>, wparam: WPARAM, lparam: LPARAM) -> bool {
        let Some(pic) = pic else { return false };
        let Some(key) = self.key_of(wparam, lparam) else {
            return false;
        };
        if self.blocked(pic) {
            return false;
        }
        let answer = self.sh.link.borrow_mut().call(&Request::Key { key });
        let Some(Response::State {
            handled,
            commit,
            preedit,
            cursor,
            candidates,
            ..
        }) = answer
        else {
            // No usable answer: pass the key through, and do not leave a
            // stale composition on screen.
            if self.sh.comp.borrow().is_some() {
                cancel_composition(&self.sh);
            }
            return false;
        };
        let view = View::from_parts(&preedit, cursor, candidates.as_ref());
        if !(commit.is_empty() && view == *self.sh.last.borrow()) {
            let es: ITfEditSession = ApplySession {
                sh: self.sh.clone(),
                ctx: pic.clone(),
                commit,
                view,
                _live: Live::new(),
            }
            .into();
            request(&self.sh, pic, &es, true, true);
        }
        if handled {
            self.sh.eaten.set(wparam.0 as u32);
        }
        handled
    }

    fn reset_all(&self) {
        cancel_composition(&self.sh);
        self.sh.send_reset();
    }
}

impl Drop for TextService {
    fn drop(&mut self) {}
}

fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(fallback)
}

fn guard_result(f: impl FnOnce() -> Result<()>) -> Result<()> {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| Err(E_FAIL.into()))
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        guard_result(|| self.activate(ptim, tid))
    }

    fn Deactivate(&self) -> Result<()> {
        guard_result(|| {
            self.deactivate();
            Ok(())
        })
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Option<&ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        guard_result(|| self.activate(ptim, tid))
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(&self, pic: Option<&ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL::from(guard(false, || self.test_key(pic, wparam, lparam))))
    }

    fn OnTestKeyUp(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        // Consume the release of a key whose press we consumed; both the test
        // and the real call read the same state.
        Ok(BOOL::from(self.sh.eaten.get() == wparam.0 as u32))
    }

    fn OnKeyDown(&self, pic: Option<&ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL::from(guard(false, || self.do_key(pic, wparam, lparam))))
    }

    fn OnKeyUp(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        let eaten = self.sh.eaten.get() == vk;
        if eaten {
            self.sh.eaten.set(0);
        }
        Ok(BOOL::from(eaten))
    }

    fn OnPreservedKey(&self, _pic: Option<&ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        Ok(BOOL(0))
    }
}

impl ITfThreadMgrEventSink_Impl for TextService_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Option<&ITfDocumentMgr>) -> Result<()> {
        guard_result(|| {
            self.sh.scope.set(None);
            self.reset_all();
            Ok(())
        })
    }

    fn OnSetFocus(
        &self,
        _focus: Option<&ITfDocumentMgr>,
        _prev: Option<&ITfDocumentMgr>,
    ) -> Result<()> {
        guard_result(|| {
            self.sh.scope.set(None);
            self.sh.eaten.set(0);
            self.reset_all();
            Ok(())
        })
    }

    fn OnPushContext(&self, _pic: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }

    fn OnPopContext(&self, _pic: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }
}

impl ITfDisplayAttributeProvider_Impl for TextService_Impl {
    fn EnumDisplayAttributeInfo(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(DaEnum::new())
    }

    fn GetDisplayAttributeInfo(&self, guid: *const GUID) -> Result<ITfDisplayAttributeInfo> {
        if guid.is_null() {
            return Err(E_INVALIDARG.into());
        }
        DaInfo::for_guid(unsafe { &*guid }).ok_or_else(|| E_INVALIDARG.into())
    }
}
