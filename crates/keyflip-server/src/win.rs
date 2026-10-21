//! Win32 glue: user SID, pipe security, pipe instances, single instance and stop event.

use std::ffi::c_void;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_ALREADY_EXISTS, ERROR_PIPE_CONNECTED, HANDLE,
    INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenUser, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY,
    TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, CreateMutexW, GetCurrentProcess, OpenEventW, OpenProcessToken, SetEvent,
    WaitForSingleObject, EVENT_MODIFY_STATE, INFINITE,
};

use keyflip_proto::{pipe_name, read_frame, write_frame, Shared};

const MUTEX_NAME: &str = "Local\\keyflip-server-mutex";
const STOP_EVENT_NAME: &str = "Local\\keyflip-server-stop";
const LOG_CAP: u64 = 256 * 1024;
const PIPE_BUF: u32 = 64 * 1024;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn last_error() -> io::Error {
    io::Error::last_os_error()
}

/// String form of the user SID of the current process token.
pub fn user_sid_string() -> io::Result<String> {
    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(last_error());
        }
        let result = (|| {
            let mut need = 0u32;
            GetTokenInformation(token, TokenUser, null_mut(), 0, &mut need);
            if need == 0 {
                return Err(last_error());
            }
            // u64 elements keep the buffer aligned for TOKEN_USER.
            let mut buf = vec![0u64; (need as usize).div_ceil(8)];
            if GetTokenInformation(token, TokenUser, buf.as_mut_ptr().cast(), need, &mut need) == 0 {
                return Err(last_error());
            }
            let user = &*(buf.as_ptr() as *const TOKEN_USER);
            let mut p: *mut u16 = null_mut();
            if ConvertSidToStringSidW(user.User.Sid, &mut p) == 0 {
                return Err(last_error());
            }
            let mut len = 0;
            while *p.add(len) != 0 {
                len += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
            LocalFree(p as *mut c_void);
            Ok(s)
        })();
        CloseHandle(token);
        result
    }
}

pub fn pipe_path() -> io::Result<String> {
    Ok(pipe_name(&user_sid_string()?))
}

pub fn data_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join("keyflip"))
}

static LOG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Create the data dir and cap the log. Call once at startup.
pub fn init_log() {
    let path = data_dir().map(|d| {
        let _ = std::fs::create_dir_all(&d);
        d.join("server.log")
    });
    if let Some(p) = &path {
        if std::fs::metadata(p).map(|m| m.len() > LOG_CAP).unwrap_or(false) {
            let _ = File::create(p);
        }
    }
    let _ = LOG_PATH.set(path);
}

/// Append one line to server.log. Never fails.
pub fn log(msg: &str) {
    if let Some(Some(p)) = LOG_PATH.get() {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{msg}");
        }
    }
}

pub fn load_options() -> keyflip_core::Options {
    let text = data_dir()
        .and_then(|d| std::fs::read_to_string(d.join("config.json")).ok())
        .unwrap_or_default();
    crate::config::parse_options(&text)
}

/// Security descriptor for the pipe: current user full control, app containers
/// read/write, mandatory label low so low-integrity clients may write.
struct PipeSecurity {
    attrs: SECURITY_ATTRIBUTES,
}

// The descriptor is allocated once, never freed, and only read.
unsafe impl Send for PipeSecurity {}
unsafe impl Sync for PipeSecurity {}

impl PipeSecurity {
    fn new(user_sid: &str) -> io::Result<PipeSecurity> {
        let sddl = format!(
            "D:P(A;;GA;;;{user_sid})(A;;GRGW;;;S-1-15-2-1)(A;;GRGW;;;S-1-15-2-2)S:(ML;;NW;;;LW)"
        );
        let w = wide(&sddl);
        let mut psd: PSECURITY_DESCRIPTOR = null_mut();
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(w.as_ptr(), 1, &mut psd, null_mut())
        };
        if ok == 0 {
            return Err(last_error());
        }
        Ok(PipeSecurity {
            attrs: SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: psd,
                bInheritHandle: 0,
            },
        })
    }
}

fn create_instance(name: &[u16], first: bool, sec: &PipeSecurity) -> io::Result<File> {
    let mut open_mode = PIPE_ACCESS_DUPLEX;
    if first {
        // Refuse to run if somebody else already owns this pipe name.
        open_mode |= FILE_FLAG_FIRST_PIPE_INSTANCE;
    }
    let h = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            open_mode,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_UNLIMITED_INSTANCES,
            PIPE_BUF,
            PIPE_BUF,
            0,
            &sec.attrs,
        )
    };
    if h == INVALID_HANDLE_VALUE {
        return Err(last_error());
    }
    Ok(unsafe { File::from_raw_handle(h as RawHandle) })
}

fn connect(pipe: &File) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    let ok = unsafe { ConnectNamedPipe(pipe.as_raw_handle() as HANDLE, null_mut()) };
    if ok == 0 && unsafe { GetLastError() } != ERROR_PIPE_CONNECTED {
        return Err(last_error());
    }
    Ok(())
}

fn serve_connection(mut pipe: File, shared: Shared, opt: keyflip_core::Options) {
    let mut session = shared.session(opt);
    loop {
        match read_frame(&mut pipe) {
            Ok(Some(payload)) => {
                let out = session.handle_payload(&payload);
                if write_frame(&mut pipe, &out).is_err() {
                    break;
                }
            }
            Ok(None) => break,
            Err(e) => {
                // A client that just goes away is normal; log anything else.
                if e.kind() != io::ErrorKind::UnexpectedEof && e.raw_os_error() != Some(109) {
                    log(&format!("connection read error: {e}"));
                }
                break;
            }
        }
    }
    // Dropping the file closes this instance, which disconnects the client.
}

/// Run the server until the stop event is signalled. Returns the process exit code.
pub fn run_server() -> i32 {
    init_log();
    std::panic::set_hook(Box::new(|info| log(&format!("panic: {info}"))));

    let mutex_name = wide(MUTEX_NAME);
    let mutex = unsafe { CreateMutexW(null(), 0, mutex_name.as_ptr()) };
    if mutex.is_null() {
        log(&format!("CreateMutex failed: {}", last_error()));
        return 1;
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return 0; // another instance is serving this user
    }

    let stop_name = wide(STOP_EVENT_NAME);
    let stop_event = unsafe { CreateEventW(null(), 1, 0, stop_name.as_ptr()) };
    if stop_event.is_null() {
        log(&format!("CreateEvent failed: {}", last_error()));
        return 1;
    }

    let sid = match user_sid_string() {
        Ok(s) => s,
        Err(e) => {
            log(&format!("cannot read user SID: {e}"));
            return 1;
        }
    };
    let path = pipe_name(&sid);
    let name = wide(&path);
    let sec = match PipeSecurity::new(&sid) {
        Ok(s) => s,
        Err(e) => {
            log(&format!("cannot build pipe security: {e}"));
            return 1;
        }
    };
    let mut next = match create_instance(&name, true, &sec) {
        Ok(p) => p,
        Err(e) => {
            log(&format!("cannot create pipe {path}: {e}"));
            return 1;
        }
    };

    let shared = Shared::bundled();
    let opt = load_options();

    static STOPPING: AtomicBool = AtomicBool::new(false);
    {
        let ev = stop_event as usize;
        let path = path.clone();
        std::thread::spawn(move || {
            unsafe { WaitForSingleObject(ev as HANDLE, INFINITE) };
            STOPPING.store(true, Ordering::SeqCst);
            // Wake the blocking ConnectNamedPipe below.
            let _ = OpenOptions::new().read(true).write(true).open(&path);
        });
    }

    loop {
        if let Err(e) = connect(&next) {
            log(&format!("ConnectNamedPipe failed: {e}"));
            std::thread::sleep(std::time::Duration::from_millis(200));
            if STOPPING.load(Ordering::SeqCst) {
                break;
            }
            // Keep serving: replace the broken instance.
            match create_instance(&name, false, &sec) {
                Ok(p) => next = p,
                Err(e) => {
                    log(&format!("cannot recreate pipe: {e}"));
                    return 1;
                }
            }
            continue;
        }
        if STOPPING.load(Ordering::SeqCst) {
            break;
        }
        let current = next;
        // Have the next instance ready before handing this one off.
        next = match create_instance(&name, false, &sec) {
            Ok(p) => p,
            Err(e) => {
                log(&format!("cannot create pipe instance: {e}"));
                return 1;
            }
        };
        let shared = shared.clone();
        let opt = opt.clone();
        std::thread::spawn(move || serve_connection(current, shared, opt));
    }
    0
}

/// `--stop`: signal a running server. Nothing running is not an error.
pub fn stop_server() -> i32 {
    let name = wide(STOP_EVENT_NAME);
    unsafe {
        let ev = OpenEventW(EVENT_MODIFY_STATE, 0, name.as_ptr());
        if !ev.is_null() {
            SetEvent(ev);
            CloseHandle(ev);
        }
    }
    0
}

/// Client side: open the pipe, retrying briefly while the server starts or is busy.
pub fn connect_client(timeout: std::time::Duration) -> io::Result<File> {
    let path = pipe_path()?;
    let start = std::time::Instant::now();
    loop {
        match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(f) => return Ok(f),
            Err(e) => {
                // 2 = not found (server starting), 231 = all instances busy.
                let retry = matches!(e.raw_os_error(), Some(2) | Some(231));
                if !retry || start.elapsed() > timeout {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}
