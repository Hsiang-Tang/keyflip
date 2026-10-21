//! Identifiers of the keyflip text service. Mirrored in `specs/002-platform-shells/windows.md`.
//! Plain numbers so the module compiles on every platform.

/// COM class of the text service.
pub const CLSID_U128: u128 = 0xee404856_d919_44a1_9e03_44a3a9687088;
/// Language profile of the text service.
pub const PROFILE_U128: u128 = 0x7a5720d5_9112_4ceb_aad3_d59ab153fbe8;
/// Display attribute for text being composed (underline).
pub const DA_INPUT_U128: u128 = 0xa486b672_f12c_4286_bb0d_92fa6a8e7cde;
/// Display attribute for the selected segment (highlight).
pub const DA_SELECTED_U128: u128 = 0x9a5bfb0c_829b_442c_9d86_32383e39963e;

/// zh-TW.
pub const LANGID_ZH_TW: u16 = 0x0404;
/// Name shown in the Windows language list.
pub const DISPLAY_NAME: &str = "keyflip 注音";
/// File names next to the DLL.
pub const SERVER_EXE: &str = "keyflip-server.exe";

/// Round trip budget to the engine server, in milliseconds.
pub const ROUND_TRIP_MS: u32 = 1000;
/// After a failed round trip, wait this long before talking to the server again.
pub const RETRY_BACKOFF_MS: u64 = 2000;
/// Do not try to start the server more often than this.
pub const SPAWN_BACKOFF_MS: u64 = 10_000;
