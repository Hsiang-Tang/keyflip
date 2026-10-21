//! Windows-only part: COM class, registration, DLL exports.

mod candwin;
mod dattr;
mod tip;

use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

use windows::core::{implement, Interface, Result, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    BOOL, CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_POINTER,
    HMODULE, S_FALSE, S_OK, TRUE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IClassFactory, IClassFactory_Impl,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::LibraryLoader::DisableThreadLibraryCalls;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_CLASSES_ROOT,
    KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, ITfCategoryMgr,
    ITfInputProcessorProfileMgr, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
    GUID_TFCAT_TIPCAP_COMLESS, GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT, GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED, GUID_TFCAT_TIP_KEYBOARD,
};

use crate::consts::{CLSID_U128, DISPLAY_NAME, LANGID_ZH_TW, PROFILE_U128};

static MODULE: AtomicIsize = AtomicIsize::new(0);
static OBJECTS: AtomicUsize = AtomicUsize::new(0);
static LOCKS: AtomicUsize = AtomicUsize::new(0);

pub fn module() -> HMODULE {
    HMODULE(MODULE.load(Ordering::SeqCst) as *mut c_void)
}

/// Counts every live COM object whose code lives in this DLL, so that
/// `DllCanUnloadNow` is honest.
pub struct Live;

impl Live {
    pub fn new() -> Live {
        OBJECTS.fetch_add(1, Ordering::SeqCst);
        Live
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        OBJECTS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn clsid() -> GUID {
    GUID::from_u128(CLSID_U128)
}

pub fn profile_guid() -> GUID {
    GUID::from_u128(PROFILE_U128)
}

#[no_mangle]
pub extern "system" fn DllMain(hinst: HMODULE, reason: u32, _reserved: *mut c_void) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        MODULE.store(hinst.0 as isize, Ordering::SeqCst);
        unsafe {
            let _ = DisableThreadLibraryCalls(hinst);
        }
    }
    TRUE
}

#[implement(IClassFactory)]
struct Factory {
    _live: Live,
}

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        outer: Option<&windows::core::IUnknown>,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> Result<()> {
        if ppv.is_null() || riid.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { *ppv = std::ptr::null_mut() };
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        catch_unwind(AssertUnwindSafe(|| {
            let obj: windows::core::IUnknown = tip::TextService::new().into();
            unsafe { obj.query(riid, ppv).ok() }
        }))
        .unwrap_or_else(|_| Err(E_FAIL.into()))
    }

    fn LockServer(&self, lock: BOOL) -> Result<()> {
        if lock.as_bool() {
            LOCKS.fetch_add(1, Ordering::SeqCst);
        } else {
            if LOCKS.load(Ordering::SeqCst) > 0 {
                LOCKS.fetch_sub(1, Ordering::SeqCst);
            }
        }
        Ok(())
    }
}

#[no_mangle]
pub extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    catch_unwind(AssertUnwindSafe(|| {
        if rclsid.is_null() || riid.is_null() || ppv.is_null() {
            return E_POINTER;
        }
        unsafe {
            *ppv = std::ptr::null_mut();
            if *rclsid != clsid() {
                return CLASS_E_CLASSNOTAVAILABLE;
            }
            let f: IClassFactory = Factory { _live: Live::new() }.into();
            f.query(riid, ppv)
        }
    }))
    .unwrap_or(E_FAIL)
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if OBJECTS.load(Ordering::SeqCst) == 0 && LOCKS.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn braced(g: &GUID) -> String {
    format!("{{{g:?}}}").to_uppercase()
}

unsafe fn set_str(sub: &str, name: Option<&str>, value: &str) -> Result<()> {
    let sub_w = wide(sub);
    let mut key = HKEY::default();
    RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        PCWSTR(sub_w.as_ptr()),
        0,
        PCWSTR::null(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        None,
        &mut key,
        None,
    )
    .ok()?;
    let v = wide(value);
    let bytes = std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 2);
    let name_w = name.map(wide);
    let name_p = name_w
        .as_ref()
        .map_or(PCWSTR::null(), |n| PCWSTR(n.as_ptr()));
    let r = RegSetValueExW(key, name_p, 0, REG_SZ, Some(bytes));
    let _ = RegCloseKey(key);
    r.ok()
}

fn categories() -> [GUID; 7] {
    [
        GUID_TFCAT_TIP_KEYBOARD,
        GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
        GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
        GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
        GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,
        GUID_TFCAT_TIPCAP_COMLESS,
        GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
    ]
}

/// Keeps COM initialised for the duration of registration.
struct ComInit(bool);

impl ComInit {
    fn new() -> ComInit {
        // S_OK and S_FALSE both need a matching CoUninitialize; a thread that
        // is already initialised another way (RPC_E_CHANGED_MODE) does not.
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        ComInit(hr.is_ok())
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

fn register() -> Result<()> {
    let dll = crate::pipe::module_path(module()).ok_or(windows::core::Error::from(E_FAIL))?;
    let id = clsid();
    let key = format!("CLSID\\{}", braced(&id));
    unsafe {
        set_str(&key, None, DISPLAY_NAME)?;
        let inproc = format!("{key}\\InprocServer32");
        set_str(&inproc, None, &dll)?;
        set_str(&inproc, Some("ThreadingModel"), "Apartment")?;

        let _com = ComInit::new();
        let profiles: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        let desc: Vec<u16> = DISPLAY_NAME.encode_utf16().collect();
        // hklsubstitute stays NULL: a value here makes Win32 windows flash
        // back to English.
        profiles.RegisterProfile(
            &id,
            LANGID_ZH_TW,
            &profile_guid(),
            &desc,
            &[],
            0,
            windows::Win32::UI::Input::KeyboardAndMouse::HKL(std::ptr::null_mut()),
            0,
            TRUE,
            0,
        )?;
        let cats: ITfCategoryMgr = CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for c in categories() {
            cats.RegisterCategory(&id, &c, &id)?;
        }
    }
    Ok(())
}

fn unregister() -> Result<()> {
    let id = clsid();
    unsafe {
        let _com = ComInit::new();
        if let Ok(cats) =
            CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
        {
            for c in categories() {
                let _ = cats.UnregisterCategory(&id, &c, &id);
            }
        }
        if let Ok(profiles) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = profiles.UnregisterProfile(&id, LANGID_ZH_TW, &profile_guid(), 0);
        }
        let sub = wide(&format!("CLSID\\{}", braced(&id)));
        let _ = RegDeleteTreeW(HKEY_CLASSES_ROOT, PCWSTR(sub.as_ptr()));
    }
    Ok(())
}

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    catch_unwind(AssertUnwindSafe(|| match register() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    }))
    .unwrap_or(E_FAIL)
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    catch_unwind(AssertUnwindSafe(|| match unregister() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    }))
    .unwrap_or(E_FAIL)
}

