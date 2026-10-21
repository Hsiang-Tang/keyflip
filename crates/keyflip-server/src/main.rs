#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    let code = if std::env::args().any(|a| a == "--stop") {
        keyflip_server::win::stop_server()
    } else {
        keyflip_server::win::run_server()
    };
    std::process::exit(code);
}

#[cfg(not(windows))]
fn main() {
    println!("windows only");
}
