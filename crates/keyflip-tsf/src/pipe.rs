//! Client side of the engine pipe: overlapped I/O with a hard deadline, so a
//! stuck server can never hang the application's UI thread.

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use keyflip_proto::{pipe_name, write_frame, Request, Response, MAX_FRAME};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, LocalFree, ERROR_FILE_NOT_FOUND, ERROR_IO_PENDING, HANDLE, HLOCAL, HMODULE,
    WAIT_OBJECT_0,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, FILE_SHARE_NONE, OPEN_EXISTING,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Threading::{
    CreateEventW, CreateProcessW, GetCurrentProcess, OpenProcessToken, ResetEvent,
    WaitForSingleObject, CREATE_NO_WINDOW, DETACHED_PROCESS, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};

use crate::consts::{RETRY_BACKOFF_MS, ROUND_TRIP_MS, SERVER_EXE, SPAWN_BACKOFF_MS};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// String form of the current process's user SID.
pub fn current_user_sid() -> Option<String> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).ok()?;
        let mut need = 0u32;
        let _ = GetTokenInformation(token, TokenUser, None, 0, &mut need);
        let mut buf = vec![0u8; need.max(64) as usize];
        let got = GetTokenInformation(
            token,
            TokenUser,
            Some(buf.as_mut_ptr().cast()),
            buf.len() as u32,
            &mut need,
        );
        let _ = CloseHandle(token);
        got.ok()?;
        let user = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut s = PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut s).ok()?;
        let out = s.to_string().ok();
        let _ = LocalFree(HLOCAL(s.0.cast()));
        out
    }
}

#[derive(Debug)]
pub enum ConnectError {
    /// The pipe does not exist: the server is not running.
    NotFound,
    Other,
}

/// One open pipe to the server.
pub struct Conn {
    h: HANDLE,
    ev: HANDLE,
}

impl Drop for Conn {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.h);
            let _ = CloseHandle(self.ev);
        }
    }
}

impl Conn {
    pub fn connect(name: &str) -> Result<Conn, ConnectError> {
        unsafe {
            let w = wide(name);
            let h = CreateFileW(
                PCWSTR(w.as_ptr()),
                0xC000_0000, // GENERIC_READ | GENERIC_WRITE
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                None,
            )
            .map_err(|e| {
                if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() {
                    ConnectError::NotFound
                } else {
                    ConnectError::Other
                }
            })?;
            let ev = match CreateEventW(None, true, false, None) {
                Ok(e) => e,
                Err(_) => {
                    let _ = CloseHandle(h);
                    return Err(ConnectError::Other);
                }
            };
            Ok(Conn { h, ev })
        }
    }

    /// Run one overlapped read or write to completion or to the deadline.
    fn io(&self, write: bool, buf: &mut [u8], deadline: Instant) -> io::Result<usize> {
        unsafe {
            let _ = ResetEvent(self.ev);
            let mut ov = OVERLAPPED {
                hEvent: self.ev,
                ..Default::default()
            };
            let started = if write {
                WriteFile(self.h, Some(&*buf), None, Some(&mut ov))
            } else {
                ReadFile(self.h, Some(buf), None, Some(&mut ov))
            };
            if let Err(e) = started {
                if e.code() != ERROR_IO_PENDING.to_hresult() {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()));
                }
                let left = deadline.saturating_duration_since(Instant::now());
                let ms = left.as_millis().min(u32::MAX as u128) as u32;
                if WaitForSingleObject(self.ev, ms) != WAIT_OBJECT_0 {
                    // The buffers live on our stack: cancel and wait for the
                    // kernel to let go of them before returning.
                    let _ = CancelIoEx(self.h, Some(&ov));
                    let mut n = 0u32;
                    let _ = GetOverlappedResult(self.h, &ov, &mut n, true);
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "pipe timeout"));
                }
            }
            let mut n = 0u32;
            GetOverlappedResult(self.h, &ov, &mut n, false)
                .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()))?;
            Ok(n as usize)
        }
    }

    fn write_all(&self, data: &[u8], deadline: Instant) -> io::Result<()> {
        let mut off = 0;
        let mut owned = data.to_vec();
        while off < owned.len() {
            let n = self.io(true, &mut owned[off..], deadline)?;
            if n == 0 {
                return Err(io::ErrorKind::WriteZero.into());
            }
            off += n;
        }
        Ok(())
    }

    fn read_exact(&self, out: &mut [u8], deadline: Instant) -> io::Result<()> {
        let mut off = 0;
        while off < out.len() {
            let n = self.io(false, &mut out[off..], deadline)?;
            if n == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            off += n;
        }
        Ok(())
    }

    /// One request, one response, within `timeout_ms` in total.
    pub fn round_trip(&self, req: &Request, timeout_ms: u32) -> io::Result<Response> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
        let payload = serde_json::to_vec(req).map_err(io::Error::other)?;
        let mut frame = Vec::with_capacity(payload.len() + 4);
        write_frame(&mut frame, &payload)?;
        self.write_all(&frame, deadline)?;
        let mut len = [0u8; 4];
        self.read_exact(&mut len, deadline)?;
        let len = u32::from_le_bytes(len) as usize;
        if len > MAX_FRAME {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut body = vec![0u8; len];
        self.read_exact(&mut body, deadline)?;
        serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Directory that holds this DLL (and `keyflip-server.exe` next to it).
pub fn module_dir(module: HMODULE) -> Option<PathBuf> {
    unsafe {
        let mut buf = [0u16; 520];
        let n = GetModuleFileNameW(module, &mut buf) as usize;
        if n == 0 || n >= buf.len() {
            return None;
        }
        let p = PathBuf::from(String::from_utf16_lossy(&buf[..n]));
        p.parent().map(|d| d.to_path_buf())
    }
}

/// Path of the DLL itself.
pub fn module_path(module: HMODULE) -> Option<String> {
    unsafe {
        let mut buf = [0u16; 520];
        let n = GetModuleFileNameW(module, &mut buf) as usize;
        if n == 0 || n >= buf.len() {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..n]))
    }
}

/// Try to start the server. Failure is ignored: sandboxed apps cannot spawn,
/// and the installer registers autostart anyway.
fn spawn_server(dir: &std::path::Path) {
    let exe = dir.join(SERVER_EXE);
    if !exe.exists() {
        return;
    }
    unsafe {
        let exe_w = wide(&exe.to_string_lossy());
        let mut cmd = wide(&format!("\"{}\"", exe.to_string_lossy()));
        let dir_w = wide(&dir.to_string_lossy());
        let si = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        if CreateProcessW(
            PCWSTR(exe_w.as_ptr()),
            PWSTR(cmd.as_mut_ptr()),
            None,
            None,
            false,
            CREATE_NO_WINDOW | DETACHED_PROCESS,
            None,
            PCWSTR(dir_w.as_ptr()),
            &si,
            &mut pi,
        )
        .is_ok()
        {
            let _ = CloseHandle(pi.hProcess);
            let _ = CloseHandle(pi.hThread);
        }
    }
}

/// A lazily connected, self-healing link to the server. Every failure means
/// "no answer"; callers then pass keys through.
pub struct Link {
    conn: Option<Conn>,
    pipe: Option<String>,
    dir: Option<PathBuf>,
    next_try: Option<Instant>,
    last_spawn: Option<Instant>,
}

impl Link {
    pub fn new(module: HMODULE) -> Link {
        Link {
            conn: None,
            pipe: current_user_sid().map(|s| pipe_name(&s)),
            dir: module_dir(module),
            next_try: None,
            last_spawn: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    fn back_off(&mut self) {
        self.conn = None;
        self.next_try = Some(Instant::now() + Duration::from_millis(RETRY_BACKOFF_MS));
    }

    fn ensure(&mut self) -> bool {
        if self.conn.is_some() {
            return true;
        }
        if let Some(t) = self.next_try {
            if Instant::now() < t {
                return false;
            }
        }
        let Some(name) = self.pipe.clone() else {
            return false;
        };
        match Conn::connect(&name) {
            Ok(c) => {
                self.conn = Some(c);
                true
            }
            Err(ConnectError::NotFound) => {
                let due = self
                    .last_spawn
                    .map_or(true, |t| t.elapsed() >= Duration::from_millis(SPAWN_BACKOFF_MS));
                if due {
                    self.last_spawn = Some(Instant::now());
                    if let Some(d) = self.dir.clone() {
                        spawn_server(&d);
                    }
                }
                self.back_off();
                false
            }
            Err(ConnectError::Other) => {
                self.back_off();
                false
            }
        }
    }

    /// Send one request. `None` when the server cannot be reached in time.
    pub fn call(&mut self, req: &Request) -> Option<Response> {
        if !self.ensure() {
            return None;
        }
        let conn = self.conn.as_ref()?;
        match conn.round_trip(req, ROUND_TRIP_MS) {
            Ok(r) => Some(r),
            Err(_) => {
                self.back_off();
                None
            }
        }
    }

    /// Like `call`, but never tries to connect or start the server.
    pub fn call_if_connected(&mut self, req: &Request) -> Option<Response> {
        if self.conn.is_none() {
            return None;
        }
        self.call(req)
    }
}
