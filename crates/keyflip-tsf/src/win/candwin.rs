//! The candidate window: a small own-drawn top-most popup that never takes
//! focus. Plain GDI, no Direct2D.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, InvalidateRect, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    FrameRect, GetDC, GetDeviceCaps, GetMonitorInfoW, GetSysColor, GetSysColorBrush,
    GetTextExtentPoint32W, MonitorFromPoint, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
    COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWFRAME, COLOR_WINDOWTEXT,
    DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HDC, HFONT, LOGPIXELSY,
    MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW,
    RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, UnregisterClassW,
    CREATESTRUCTW, GWLP_USERDATA, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE,
    WM_ERASEBKGND, WM_MOUSEACTIVATE, WM_NCCREATE, WM_PAINT, WNDCLASSEXW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, MA_NOACTIVATE,
};

use crate::render::CandView;

const CLASS: PCWSTR = w!("KeyflipCandidateWindow");
static WINDOWS: AtomicUsize = AtomicUsize::new(0);

const PAD_X: i32 = 8;
const PAD_Y: i32 = 4;

struct Data {
    rows: Vec<String>,
    highlight: Option<usize>,
    font: HFONT,
    row_h: i32,
}

pub struct CandWindow {
    hwnd: HWND,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

impl CandWindow {
    pub fn new() -> Option<CandWindow> {
        unsafe {
            let hinst = GetModuleHandleW(None).ok()?;
            let hinst = windows::Win32::Foundation::HINSTANCE(hinst.0);
            if WINDOWS.load(Ordering::SeqCst) == 0 {
                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    lpfnWndProc: Some(wnd_proc),
                    hInstance: hinst,
                    lpszClassName: CLASS,
                    ..Default::default()
                };
                // Zero means "already registered", which is fine too.
                let _ = RegisterClassExW(&wc);
            }
            let data = Box::new(Data {
                rows: Vec::new(),
                highlight: None,
                font: make_font(),
                row_h: 20,
            });
            let ptr = Box::into_raw(data);
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                CLASS,
                w!(""),
                WS_POPUP,
                0,
                0,
                10,
                10,
                None,
                None,
                hinst,
                Some(ptr.cast()),
            );
            match hwnd {
                Ok(hwnd) => {
                    WINDOWS.fetch_add(1, Ordering::SeqCst);
                    Some(CandWindow { hwnd })
                }
                Err(_) => {
                    // The window may already own the data (freed on WM_NCDESTROY).
                    let _ = ptr;
                    None
                }
            }
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Show `view` below `anchor` (screen coordinates), or above it when there
    /// is no room below.
    pub fn show(&self, view: &CandView, anchor: RECT) {
        unsafe {
            let p = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut Data;
            if p.is_null() {
                return;
            }
            let d = &mut *p;
            d.rows = view.rows.clone();
            d.highlight = view.highlight;

            let dc = GetDC(self.hwnd);
            let old = SelectObject(dc, d.font);
            let mut max_w = 0;
            let mut line_h = 16;
            for r in &d.rows {
                let u = wide(r);
                let mut sz = SIZE::default();
                let _ = GetTextExtentPoint32W(dc, &u, &mut sz);
                max_w = max_w.max(sz.cx);
                line_h = line_h.max(sz.cy);
            }
            SelectObject(dc, old);
            ReleaseDC(self.hwnd, dc);

            d.row_h = line_h + 2;
            let width = max_w + PAD_X * 2;
            let height = d.row_h * d.rows.len() as i32 + PAD_Y * 2;

            let mon = MonitorFromPoint(
                windows::Win32::Foundation::POINT {
                    x: anchor.left,
                    y: anchor.bottom,
                },
                MONITOR_DEFAULTTONEAREST,
            );
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let mut x = anchor.left;
            let mut y = anchor.bottom + 2;
            if GetMonitorInfoW(mon, &mut mi).as_bool() {
                let wa = mi.rcWork;
                if y + height > wa.bottom {
                    y = anchor.top - height - 2;
                }
                if x + width > wa.right {
                    x = wa.right - width;
                }
                x = x.max(wa.left);
                y = y.max(wa.top);
            }
            let _ = SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            let _ = InvalidateRect(self.hwnd, None, true);
        }
    }
}

impl Drop for CandWindow {
    fn drop(&mut self) {
        unsafe {
            // WM_NCDESTROY handling frees the data; see wnd_proc.
            let _ = DestroyWindow(self.hwnd);
            if WINDOWS.fetch_sub(1, Ordering::SeqCst) == 1 {
                if let Ok(h) = GetModuleHandleW(None) {
                    let _ = UnregisterClassW(CLASS, windows::Win32::Foundation::HINSTANCE(h.0));
                }
            }
        }
    }
}

unsafe fn make_font() -> HFONT {
    let dc = GetDC(None);
    let dpi = GetDeviceCaps(dc, LOGPIXELSY).max(96);
    ReleaseDC(None, dc);
    let height = -(11 * dpi / 72);
    CreateFontW(
        height,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        windows::Win32::Graphics::Gdi::DEFAULT_CHARSET.0 as u32,
        windows::Win32::Graphics::Gdi::OUT_DEFAULT_PRECIS.0 as u32,
        windows::Win32::Graphics::Gdi::CLIP_DEFAULT_PRECIS.0 as u32,
        windows::Win32::Graphics::Gdi::CLEARTYPE_QUALITY.0 as u32,
        0,
        w!("Microsoft JhengHei UI"),
    )
}

unsafe fn paint(hwnd: HWND, d: &Data) {
    let mut ps = PAINTSTRUCT::default();
    let dc: HDC = BeginPaint(hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client);
    FillRect(dc, &client, GetSysColorBrush(COLOR_WINDOW));
    let frame = CreateSolidBrush(COLORREF(GetSysColor(COLOR_WINDOWFRAME)));
    FrameRect(dc, &client, frame);
    let _ = DeleteObject(frame);

    let old = SelectObject(dc, d.font);
    SetBkMode(dc, TRANSPARENT);
    for (i, row) in d.rows.iter().enumerate() {
        let top = PAD_Y + d.row_h * i as i32;
        let mut rc = RECT {
            left: 2,
            top,
            right: client.right - 2,
            bottom: top + d.row_h,
        };
        if d.highlight == Some(i) {
            FillRect(dc, &rc, GetSysColorBrush(COLOR_HIGHLIGHT));
            SetTextColor(dc, COLORREF(GetSysColor(COLOR_HIGHLIGHTTEXT)));
        } else {
            SetTextColor(dc, COLORREF(GetSysColor(COLOR_WINDOWTEXT)));
        }
        rc.left = PAD_X;
        let mut u = wide(row);
        DrawTextW(dc, &mut u, &mut rc, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
    }
    SelectObject(dc, old);
    let _ = EndPaint(hwnd, &ps);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let r = catch_unwind(AssertUnwindSafe(|| match msg {
        WM_NCCREATE => {
            let cs = &*(lp.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Data;
            if !p.is_null() {
                paint(hwnd, &*p);
            }
            LRESULT(0)
        }
        // WM_NCDESTROY
        0x0082 => {
            let p = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut Data;
            if !p.is_null() {
                let d = Box::from_raw(p);
                let _ = DeleteObject(d.font);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }));
    r.unwrap_or(LRESULT(0))
}
