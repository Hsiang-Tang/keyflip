//! Diagnostic client for the engine pipe.
//!
//! `keyflip-pipe-client` sends a ping. `keyflip-pipe-client '{"op":"reset"}'`
//! sends any request as JSON and prints the response. Exit code 0 means the
//! server answered, 1 means it did not.

#[cfg(windows)]
fn main() {
    use keyflip_proto::{pipe_name, Request};
    use keyflip_tsf::pipe::{current_user_sid, Conn};

    let req: Request = match std::env::args().nth(1) {
        Some(j) => match serde_json::from_str(&j) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("bad request: {e}");
                std::process::exit(2);
            }
        },
        None => Request::Ping,
    };
    let Some(sid) = current_user_sid() else {
        eprintln!("cannot read the user SID");
        std::process::exit(1);
    };
    let name = pipe_name(&sid);
    let conn = match Conn::connect(&name) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cannot open {name}: {e:?}");
            std::process::exit(1);
        }
    };
    match conn.round_trip(&req, 2000) {
        Ok(resp) => println!("{}", serde_json::to_string(&resp).unwrap_or_default()),
        Err(e) => {
            eprintln!("no answer: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("keyflip-pipe-client only runs on Windows");
    std::process::exit(1);
}
