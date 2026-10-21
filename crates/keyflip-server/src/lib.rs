//! keyflip engine server: serves `keyflip-proto` sessions over a per-user named pipe.
//!
//! Everything pure (config parsing, key notation mapping) is platform independent;
//! the Win32 glue lives in [`win`] and only builds on Windows.

pub mod config;
pub mod notation;
#[cfg(windows)]
pub mod win;
