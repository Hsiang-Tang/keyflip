//! `keyflip-tsf-smoke <path-to-keyflip_tsf.dll>`: load the DLL like a host would,
//! without registering anything. Needs no administrator rights.
//!
//! Checks that every import resolves, that the four COM exports exist, that the
//! class factory can be obtained for our CLSID and can create the text service
//! object, and that the DLL unloads once everything is released.

#[cfg(not(windows))]
fn main() {
    eprintln!("windows only");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() {
    use std::ffi::c_void;

    use keyflip_tsf::consts::CLSID_U128;
    use windows::core::{s, w, Interface, GUID, HRESULT, PCWSTR};
    use windows::Win32::Foundation::{FreeLibrary, S_FALSE, S_OK};
    use windows::Win32::System::Com::IClassFactory;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows::Win32::UI::TextServices::ITfTextInputProcessor;

    let _ = w!("");
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: keyflip-tsf-smoke <dll>");
            std::process::exit(2);
        }
    };
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let module = match unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) } {
        Ok(m) => m,
        Err(e) => {
            eprintln!("FAIL LoadLibrary: {e}");
            std::process::exit(1);
        }
    };
    println!("ok   loaded {path}");

    type GetClassObject = unsafe extern "system" fn(*const GUID, *const GUID, *mut *mut c_void) -> HRESULT;
    type CanUnload = unsafe extern "system" fn() -> HRESULT;
    for name in [s!("DllRegisterServer"), s!("DllUnregisterServer"), s!("DllGetClassObject"), s!("DllCanUnloadNow")] {
        if unsafe { GetProcAddress(module, name) }.is_none() {
            eprintln!("FAIL missing export {}", unsafe { name.to_string().unwrap_or_default() });
            std::process::exit(1);
        }
    }
    println!("ok   four COM exports present");

    let get: GetClassObject = unsafe { std::mem::transmute(GetProcAddress(module, s!("DllGetClassObject")).unwrap()) };
    let can_unload: CanUnload = unsafe { std::mem::transmute(GetProcAddress(module, s!("DllCanUnloadNow")).unwrap()) };

    let clsid = GUID::from_u128(CLSID_U128);
    let mut raw: *mut c_void = std::ptr::null_mut();
    let hr = unsafe { get(&clsid, &IClassFactory::IID, &mut raw) };
    if hr != S_OK || raw.is_null() {
        eprintln!("FAIL DllGetClassObject: {hr:?}");
        std::process::exit(1);
    }
    let factory = unsafe { IClassFactory::from_raw(raw) };
    println!("ok   class factory for our CLSID");

    let bogus = GUID::from_u128(0x1234_5678_9abc_def0_1234_5678_9abc_def0);
    let mut raw2: *mut c_void = std::ptr::null_mut();
    let hr2 = unsafe { get(&bogus, &IClassFactory::IID, &mut raw2) };
    if hr2 == S_OK {
        eprintln!("FAIL a foreign CLSID was accepted");
        std::process::exit(1);
    }
    println!("ok   foreign CLSID refused ({hr2:?})");

    let tip: ITfTextInputProcessor = match unsafe { factory.CreateInstance(None) } {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL CreateInstance: {e}");
            std::process::exit(1);
        }
    };
    println!("ok   text service object created");
    let busy = unsafe { can_unload() };
    if busy != S_FALSE {
        eprintln!("FAIL DllCanUnloadNow should be S_FALSE while objects live, got {busy:?}");
        std::process::exit(1);
    }
    drop(tip);
    drop(factory);
    let idle = unsafe { can_unload() };
    if idle != S_OK {
        eprintln!("FAIL DllCanUnloadNow should be S_OK after release, got {idle:?}");
        std::process::exit(1);
    }
    println!("ok   unloads cleanly after release");
    unsafe {
        let _ = FreeLibrary(module);
    }
    println!("PASS");
}
