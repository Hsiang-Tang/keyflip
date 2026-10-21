//! C interface to the keyflip engine for shells written in other languages
//! (the macOS input method). The JSON documents are exactly keyflip-proto's.

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use keyflip_core::Options;
use keyflip_proto::{Session, Shared};

mod user_store;
use user_store::UserStore;

static SHARED: OnceLock<Shared> = OnceLock::new();
static USER: OnceLock<Mutex<UserStore>> = OnceLock::new();

fn with_user(f: impl FnOnce(&mut UserStore)) {
    if let Some(store) = USER.get() {
        let mut store = store.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut store);
    }
}

pub struct KeyflipSession {
    inner: Session,
}

fn error_response(message: &str) -> *mut c_char {
    to_c(format!(
        "{{\"op\":\"error\",\"message\":{}}}",
        json_escape(message)
    ))
}

fn json_escape(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn to_c(s: String) -> *mut c_char {
    CString::new(s.replace('\0', ""))
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// Standard C ownership rules: free the result with `keyflip_session_free`.
#[no_mangle]
pub extern "C" fn keyflip_session_new() -> *mut KeyflipSession {
    catch_unwind(|| {
        let shared = SHARED.get_or_init(|| {
            let shared = Shared::bundled();
            match USER.get() {
                Some(store) => {
                    let model = store.lock().unwrap_or_else(|e| e.into_inner()).model.clone();
                    shared.with_user(model)
                }
                None => shared,
            }
        });
        Box::into_raw(Box::new(KeyflipSession {
            inner: shared.session(Options::default()),
        }))
    })
    .unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// `session` must be NULL or a pointer from `keyflip_session_new`, freed once.
#[no_mangle]
pub unsafe extern "C" fn keyflip_session_free(session: *mut KeyflipSession) {
    if !session.is_null() {
        drop(Box::from_raw(session));
    }
}

/// # Safety
/// `session` must be a live session and `request_json` a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn keyflip_handle(
    session: *mut KeyflipSession,
    request_json: *const c_char,
) -> *mut c_char {
    if session.is_null() || request_json.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(request) = CStr::from_ptr(request_json).to_str() else {
        return std::ptr::null_mut();
    };
    let session = &mut *session;
    match catch_unwind(AssertUnwindSafe(|| {
        let reply = session.inner.handle_payload(request.as_bytes());
        with_user(UserStore::save_if_changed);
        reply
    })) {
        Ok(bytes) => to_c(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => error_response("engine panic"),
    }
}

/// Opens the user files in `dir`. Call once, before the first session.
///
/// # Safety
/// `dir` must be NULL or a NUL-terminated UTF-8 path.
#[no_mangle]
pub unsafe extern "C" fn keyflip_user_open(dir: *const c_char) -> i32 {
    if dir.is_null() {
        return -1;
    }
    let Ok(dir) = CStr::from_ptr(dir).to_str() else {
        return -1;
    };
    let dir = Path::new(dir).to_owned();
    match catch_unwind(move || USER.set(Mutex::new(UserStore::open(&dir))).is_ok()) {
        Ok(true) => 0,
        _ => -1,
    }
}

#[no_mangle]
pub extern "C" fn keyflip_user_reload() {
    let _ = catch_unwind(|| with_user(UserStore::reload));
}

#[no_mangle]
pub extern "C" fn keyflip_user_clear() {
    let _ = catch_unwind(|| with_user(UserStore::clear));
}

/// # Safety
/// `s` must be NULL or a string returned by `keyflip_handle`, freed once.
#[no_mangle]
pub unsafe extern "C" fn keyflip_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    unsafe fn call(session: *mut KeyflipSession, req: &str) -> String {
        let req = CString::new(req).unwrap();
        let out = keyflip_handle(session, req.as_ptr());
        assert!(!out.is_null());
        let text = CStr::from_ptr(out).to_str().unwrap().to_owned();
        keyflip_string_free(out);
        text
    }

    #[test]
    fn typing_through_the_c_interface_commits_chinese_and_english() {
        unsafe {
            let s = keyflip_session_new();
            assert!(!s.is_null());
            let mut commit = String::new();
            for k in ["s", "u", "3", "c", "l", "3"] {
                call(s, &format!(r#"{{"op":"key","key":{{"code":"{k}","text":"{k}"}}}}"#));
            }
            let out = call(s, r#"{"op":"key","key":{"code":"Enter"}}"#);
            commit.push_str(&out);
            assert!(out.contains("你好"), "{out}");
            for (k, shift) in [("h", true), ("i", false)] {
                let t = if shift { k.to_uppercase() } else { k.to_owned() };
                call(s, &format!(r#"{{"op":"key","key":{{"code":"{k}","text":"{t}","shift":{shift}}}}}"#));
            }
            let out = call(s, r#"{"op":"key","key":{"code":"Enter"}}"#);
            assert!(out.contains("Hi"), "{out}");
            keyflip_session_free(s);
        }
    }

    #[test]
    fn bad_input_never_crashes() {
        unsafe {
            assert!(keyflip_handle(std::ptr::null_mut(), std::ptr::null()).is_null());
            let s = keyflip_session_new();
            let out = call(s, "not json");
            assert!(out.contains("error"), "{out}");
            keyflip_session_free(s);
            keyflip_session_free(std::ptr::null_mut());
            keyflip_string_free(std::ptr::null_mut());
        }
    }
}
