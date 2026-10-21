//! Wire protocol between a platform shell (TSF DLL) and the engine server.
//!
//! One connection is one typing session with its own engine state. Frames are a
//! 4-byte little-endian length followed by that many bytes of UTF-8 JSON.
//! Every request gets exactly one response, in order.
//!
//! All decisions live in [`Session::handle`], which is plain Rust and tested on
//! any platform. Servers only move frames between a pipe and this function.

use std::io::{self, Read, Write};
#[cfg(feature = "engine")]
use std::sync::Arc;

#[cfg(feature = "engine")]
use keyflip_core::{
    Candidates, Engine, English, KeyCode, KeyEvent, Lexicon, Options, SegKind, Segment, Snapshot,
};
use serde::{Deserialize, Serialize};

/// Named pipe both sides use. `user_sid` is the string form of the user SID of
/// the current process token (for example `S-1-5-21-...`), so each user gets
/// their own server and nobody can talk to another user's engine.
pub fn pipe_name(user_sid: &str) -> String {
    format!(r"\\.\pipe\keyflip-v1-{user_sid}")
}

/// Largest frame either side accepts.
pub const MAX_FRAME: usize = 1 << 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    /// Name of the key: a single character for printable keys (the unshifted
    /// physical key on a US keyboard), or one of `Space Enter Tab Backspace
    /// Delete Esc Left Right Up Down Home End`.
    pub code: String,
    /// The character this press produces with the current modifiers, if any.
    #[serde(default)]
    pub text: Option<char>,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    /// Handle a key for real and return the new state.
    Key { key: Key },
    /// Would this key be consumed? Changes nothing. For TSF `OnTestKeyDown`.
    Test { key: Key },
    /// Drop the composition without committing (focus lost, input method switched).
    Reset,
    /// Commit whatever is in the composition and return it.
    Flush,
    /// Liveness check.
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireSegment {
    pub text: String,
    /// `chinese`, `latin`, `pending`, `selected` or `suggestion`.
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireCandidates {
    pub items: Vec<String>,
    pub selected: usize,
    pub page_size: usize,
    /// Grid of every candidate, `page_size` per row. Absent from older engines.
    #[serde(default)]
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Response {
    /// Result of `Key`, `Reset` and `Flush`.
    State {
        handled: bool,
        commit: String,
        preedit: Vec<WireSegment>,
        /// UTF-16 code units from the start of the preedit.
        cursor: usize,
        candidates: Option<WireCandidates>,
        full_width: bool,
    },
    /// Result of `Test`.
    Test { handled: bool },
    Pong,
    Error { message: String },
}

#[cfg(feature = "engine")]
fn code_of(name: &str) -> Option<KeyCode> {
    Some(match name {
        "Space" => KeyCode::Space,
        "Enter" => KeyCode::Enter,
        "Tab" => KeyCode::Tab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Esc" => KeyCode::Esc,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        other => {
            let mut chars = other.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => KeyCode::Char(c),
                _ => return None,
            }
        }
    })
}

#[cfg(feature = "engine")]
impl Key {
    pub fn to_event(&self) -> Result<KeyEvent, String> {
        let code = code_of(&self.code).ok_or_else(|| format!("unknown key {:?}", self.code))?;
        Ok(KeyEvent {
            code,
            text: self.text,
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            meta: self.meta,
        })
    }
}

#[cfg(feature = "engine")]
fn state_of(s: Snapshot) -> Response {
    let kind = |k: SegKind| {
        match k {
            SegKind::Chinese => "chinese",
            SegKind::Latin => "latin",
            SegKind::Pending => "pending",
            SegKind::Selected => "selected",
            SegKind::Suggestion => "suggestion",
        }
        .to_owned()
    };
    Response::State {
        handled: s.handled,
        commit: s.commit,
        preedit: s
            .preedit
            .into_iter()
            .map(|Segment { text, kind: k }| WireSegment {
                text,
                kind: kind(k),
            })
            .collect(),
        cursor: s.cursor,
        candidates: s.candidates.map(
            |Candidates {
                 items,
                 selected,
                 page_size,
                 expanded,
             }| WireCandidates {
                items,
                selected,
                page_size,
                expanded,
            },
        ),
        full_width: s.full_width,
    }
}

/// Lexicon and word list shared by every session of a server.
#[cfg(feature = "engine")]
#[derive(Clone)]
pub struct Shared {
    lex: Arc<Lexicon>,
    eng: Arc<English>,
}

#[cfg(feature = "engine")]
impl Shared {
    pub fn bundled() -> Shared {
        Shared {
            lex: Arc::new(Lexicon::bundled()),
            eng: Arc::new(English::bundled()),
        }
    }

    pub fn session(&self, opt: Options) -> Session {
        Session {
            engine: Engine::new(self.lex.clone(), self.eng.clone(), opt),
        }
    }
}

/// One connection's engine.
#[cfg(feature = "engine")]
pub struct Session {
    engine: Engine,
}

#[cfg(feature = "engine")]
impl Session {
    pub fn handle(&mut self, req: Request) -> Response {
        match req {
            Request::Key { key } => match key.to_event() {
                Ok(ev) => state_of(self.engine.handle_key(ev)),
                Err(message) => Response::Error { message },
            },
            Request::Test { key } => match key.to_event() {
                Ok(ev) => Response::Test {
                    handled: self.engine.would_handle(ev),
                },
                Err(message) => Response::Error { message },
            },
            Request::Reset => {
                self.engine.reset();
                state_of(self.engine.current())
            }
            Request::Flush => {
                let commit = if self.engine.is_composing() {
                    let ev = KeyEvent::plain(KeyCode::Enter);
                    self.engine.handle_key(ev).commit
                } else {
                    String::new()
                };
                let mut snap = self.engine.current();
                snap.handled = true;
                snap.commit = commit;
                state_of(snap)
            }
            Request::Ping => Response::Pong,
        }
    }

    /// Decode one request frame payload and produce the response frame payload.
    pub fn handle_payload(&mut self, payload: &[u8]) -> Vec<u8> {
        let resp = match serde_json::from_slice::<Request>(payload) {
            Ok(req) => self.handle(req),
            Err(e) => Response::Error {
                message: format!("bad request: {e}"),
            },
        };
        serde_json::to_vec(&resp).expect("response serializes")
    }
}

/// Write one frame.
pub fn write_frame(w: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    if payload.len() > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "frame too large"));
    }
    w.write_all(&(payload.len() as u32).to_le_bytes())?;
    w.write_all(payload)?;
    w.flush()
}

/// Read one frame. `Ok(None)` on a clean end of stream before any byte of a frame.
pub fn read_frame(r: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len = [0u8; 4];
    match r.read(&mut len[..1])? {
        0 => return Ok(None),
        _ => r.read_exact(&mut len[1..])?,
    }
    let len = u32::from_le_bytes(len) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(Some(buf))
}

#[cfg(all(test, feature = "engine"))]
mod tests {
    use super::*;

    fn key(code: &str) -> Key {
        Key {
            code: code.into(),
            text: code.chars().next().filter(|_| code.chars().count() == 1),
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    fn session() -> Session {
        Shared::bundled().session(Options::default())
    }

    fn handled(r: &Response) -> bool {
        match r {
            Response::State { handled, .. } | Response::Test { handled } => *handled,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn test_agrees_with_key_and_changes_nothing() {
        let mut s = session();
        // Idle Enter is the application's.
        assert!(!handled(&s.handle(Request::Test { key: key("Enter") })));
        assert!(!handled(&s.handle(Request::Key { key: key("Enter") })));
        for c in ["s", "u", "3"] {
            let t = s.handle(Request::Test { key: key(c) });
            let k = s.handle(Request::Key { key: key(c) });
            assert_eq!(handled(&t), handled(&k), "{c}");
        }
        // Now composing: Enter is ours, and testing it did not consume it.
        assert!(handled(&s.handle(Request::Test { key: key("Enter") })));
        let resp = s.handle(Request::Key { key: key("Enter") });
        match resp {
            Response::State { commit, .. } => assert_eq!(commit.chars().count(), 1),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn flush_commits_the_composition_and_reset_drops_it() {
        let mut s = session();
        for ch in "Hsiang".chars() {
            let mut k = key(&ch.to_ascii_lowercase().to_string());
            if ch.is_ascii_uppercase() {
                k.shift = true;
                k.text = Some(ch);
            }
            s.handle(Request::Key { key: k });
        }
        match s.handle(Request::Flush) {
            Response::State { commit, preedit, .. } => {
                assert_eq!(commit, "Hsiang");
                assert!(preedit.is_empty());
            }
            other => panic!("{other:?}"),
        }
        s.handle(Request::Key { key: key("a") });
        match s.handle(Request::Reset) {
            Response::State { commit, preedit, .. } => {
                assert!(commit.is_empty() && preedit.is_empty());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn frames_round_trip_and_bad_input_is_an_error_not_a_panic() {
        let mut s = session();
        let mut wire = Vec::new();
        write_frame(&mut wire, br#"{"op":"ping"}"#).unwrap();
        write_frame(&mut wire, br#"{"op":"nonsense"}"#).unwrap();
        write_frame(&mut wire, br#"{"op":"key","key":{"code":"NoSuchKey"}}"#).unwrap();
        let mut r = wire.as_slice();
        let mut out = Vec::new();
        while let Some(p) = read_frame(&mut r).unwrap() {
            out.push(serde_json::from_slice::<Response>(&s.handle_payload(&p)).unwrap());
        }
        assert_eq!(out[0], Response::Pong);
        assert!(matches!(out[1], Response::Error { .. }));
        assert!(matches!(out[2], Response::Error { .. }));
    }

    #[test]
    fn candidates_from_an_older_engine_read_as_not_expanded() {
        let c: WireCandidates =
            serde_json::from_str(r#"{"items":["a"],"selected":0,"page_size":9}"#).unwrap();
        assert!(!c.expanded);
    }

    #[test]
    fn pipe_name_is_per_user() {
        assert_eq!(pipe_name("S-1-5-21-1"), r"\\.\pipe\keyflip-v1-S-1-5-21-1");
    }

    #[test]
    fn oversized_frames_are_rejected() {
        let mut wire = Vec::new();
        wire.extend_from_slice(&((MAX_FRAME as u32) + 1).to_le_bytes());
        assert!(read_frame(&mut wire.as_slice()).is_err());
    }
}
