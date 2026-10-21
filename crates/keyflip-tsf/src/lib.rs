//! keyflip Windows text service (TSF input method), built as `keyflip_tsf.dll`.
//!
//! The DLL is a thin shell. It never decides anything about Chinese versus
//! English. It turns key events into `keyflip_proto` requests, sends them to
//! the per-user engine server over a named pipe, and draws the answer. It
//! holds no engine state.

pub mod consts;
pub mod keymap;
pub mod render;

#[cfg(windows)]
pub mod pipe;
#[cfg(windows)]
mod win;
