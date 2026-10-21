//! Test client: sends key notation to the server, prints each response as one JSON line.
//!
//! keyflip-pipe-client --ping
//! keyflip-pipe-client [--expect-commit TEXT] KEYS

#[cfg(not(windows))]
fn main() {
    println!("windows only");
}

#[cfg(windows)]
fn main() {
    std::process::exit(match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
    });
}

#[cfg(windows)]
fn run() -> Result<i32, String> {
    use keyflip_proto::{read_frame, write_frame, Request, Response};
    use keyflip_server::{notation::parse_wire_keys, win};
    use std::time::Duration;

    let mut ping = false;
    let mut expect: Option<String> = None;
    let mut keys: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--ping" => ping = true,
            "--expect-commit" => expect = Some(args.next().ok_or("--expect-commit needs a value")?),
            _ if keys.is_none() => keys = Some(a),
            _ => return Err(format!("unexpected argument {a:?}")),
        }
    }
    if !ping && keys.is_none() {
        return Err("usage: keyflip-pipe-client --ping | [--expect-commit TEXT] KEYS".into());
    }

    let mut requests = Vec::new();
    if ping {
        requests.push(Request::Ping);
    }
    if let Some(k) = &keys {
        for key in parse_wire_keys(k)? {
            requests.push(Request::Key { key });
        }
    }

    let mut pipe = win::connect_client(Duration::from_secs(5)).map_err(|e| format!("connect: {e}"))?;
    let mut commit = String::new();
    for req in requests {
        let payload = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
        write_frame(&mut pipe, &payload).map_err(|e| format!("write: {e}"))?;
        let reply = read_frame(&mut pipe)
            .map_err(|e| format!("read: {e}"))?
            .ok_or("server closed the connection")?;
        let text = String::from_utf8_lossy(&reply);
        println!("{}", text.trim_end());
        match serde_json::from_slice::<Response>(&reply).map_err(|e| format!("bad response: {e}"))? {
            Response::State { commit: c, .. } => commit.push_str(&c),
            Response::Error { message } => return Err(format!("server error: {message}")),
            _ => {}
        }
    }
    if let Some(want) = expect {
        if commit != want {
            eprintln!("expected commit {want:?}, got {commit:?}");
            return Ok(1);
        }
    }
    Ok(0)
}
