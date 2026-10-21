//! keyflip-cli
//!
//! - `play <keys>`: feed key notation to the engine and print every snapshot.
//! - `eval`: measure Chinese-stays-Chinese and English-detected rates.

use std::sync::Arc;

use keyflip_core::{parse_keys, Engine, English, Lexicon, Options};

fn run(engine: &mut Engine, keys: &str) -> String {
    let mut committed = String::new();
    for ev in parse_keys(keys).expect("valid notation") {
        committed.push_str(&engine.handle_key(ev).commit);
    }
    committed
}

fn play(keys: &str) {
    let events = match parse_keys(keys) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };
    let mut engine = keyflip_core::bundled_engine(Options::default());
    let mut committed = String::new();
    for ev in events {
        let snap = engine.handle_key(ev);
        committed.push_str(&snap.commit);
        let segs: Vec<String> = snap
            .preedit
            .iter()
            .map(|s| format!("{:?}:{}", s.kind, s.text))
            .collect();
        println!(
            "{:<28} handled={} commit={:?} preedit=[{}] cursor={}{}",
            format!("{:?}", ev.code),
            snap.handled,
            snap.commit,
            segs.join(" "),
            snap.cursor,
            snap.candidates
                .as_ref()
                .map(|c| format!(" cands={:?}", &c.items[..c.items.len().min(9)]))
                .unwrap_or_default()
        );
    }
    println!("committed: {committed:?}");
}

/// Keys that type one reading (tone included).
fn keys_of_reading(reading: &str) -> Option<String> {
    let mut keys = String::new();
    let mut tone = None;
    for ch in reading.chars() {
        match ch {
            'ˇ' => tone = Some('3'),
            'ˋ' => tone = Some('4'),
            'ˊ' => tone = Some('6'),
            '˙' => tone = Some('7'),
            _ => {
                let key = ('!'..='~')
                    .find(|&k| keyflip_core::zhuyin::symbol_of_key(k) == Some(ch))?;
                keys.push(key);
            }
        }
    }
    keys.push(tone.unwrap_or('_'));
    Some(keys)
}

fn eval() {
    let lex = Arc::new(Lexicon::bundled());
    let eng = Arc::new(English::bundled());
    let mut engine = Engine::new(lex.clone(), eng.clone(), Options::default());

    // Chinese: typed phrases must never come out as English.
    let mut total = 0usize;
    let mut leaked = Vec::new();
    let mut exact = 0usize;
    let mut phrases: Vec<_> = lex
        .entries()
        .filter(|e| e.readings.len() >= 2 && e.readings.len() <= 4 && e.count >= 30)
        .collect();
    phrases.sort_by(|a, b| a.text.cmp(&b.text).then_with(|| a.readings.cmp(&b.readings)));
    for e in phrases.iter().step_by(5) {
        let Some(keys) = e
            .readings
            .iter()
            .map(|r| keys_of_reading(r))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let keys = keys.concat();
        let out = run(&mut engine, &format!("{keys}<E>"));
        total += 1;
        if out == e.text {
            exact += 1;
        }
        if out.chars().any(|c| c.is_ascii_alphanumeric()) {
            leaked.push(format!("{} keys={keys} out={out}", e.text));
        }
    }
    println!(
        "Chinese phrases: {total} typed, {} leaked into English ({:.3}%), {exact} exact ({:.1}%)",
        leaked.len(),
        100.0 * leaked.len() as f64 / total as f64,
        100.0 * exact as f64 / total as f64
    );
    for l in leaked.iter().take(20) {
        println!("  leak: {l}");
    }

    // Mixed: Chinese phrase, English word, space, Chinese phrase.
    let words_all = eng.words_by_rank();
    let words: Vec<&str> = words_all
        .iter()
        .skip(100)
        .take(2900)
        .copied()
        .filter(|w| w.len() >= 3)
        .collect();
    let mut bad = Vec::new();
    let mut n = 0usize;
    for (i, w) in words.iter().enumerate() {
        let a = phrases[(i * 7) % phrases.len()];
        let b = phrases[(i * 13 + 5) % phrases.len()];
        let ka = a.readings.iter().map(|r| keys_of_reading(r)).collect::<Option<Vec<_>>>();
        let kb = b.readings.iter().map(|r| keys_of_reading(r)).collect::<Option<Vec<_>>>();
        let (Some(ka), Some(kb)) = (ka, kb) else { continue };
        let keys = format!("{}{}_{}<E>", ka.concat(), w, kb.concat());
        let out = run(&mut engine, &keys);
        n += 1;
        let ascii: String = out.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        if ascii != *w || !out.contains(&format!("{w} ")) && !out.contains(w) {
            bad.push(format!("{}+{w}+{} -> {out}", a.text, b.text));
        }
    }
    println!(
        "Mixed zh+en+zh: {n} sentences, {} wrong ({:.1}%)",
        bad.len(),
        100.0 * bad.len() as f64 / n.max(1) as f64
    );
    for b in bad.iter().take(25) {
        println!("  wrong: {b}");
    }

    // Glued: Chinese phrase, English word, Chinese phrase, no space (MI-23).
    let mut bad = Vec::new();
    let mut n = 0usize;
    for (i, w) in words.iter().enumerate() {
        let a = phrases[(i * 7) % phrases.len()];
        let b = phrases[(i * 13 + 5) % phrases.len()];
        let ka = a.readings.iter().map(|r| keys_of_reading(r)).collect::<Option<Vec<_>>>();
        let kb = b.readings.iter().map(|r| keys_of_reading(r)).collect::<Option<Vec<_>>>();
        let (Some(ka), Some(kb)) = (ka, kb) else { continue };
        let kb = kb.concat();
        // A first-tone syllable ends with Space, which ends the word anyway; skip those.
        if kb.contains('_') {
            continue;
        }
        let out = run(&mut engine, &format!("{}{}{}<E>", ka.concat(), w, kb));
        n += 1;
        let tail = out.split_once(*w).map(|(_, t)| t).unwrap_or("x");
        if tail.is_empty() || tail.chars().any(|c| c.is_ascii()) {
            bad.push(format!("{}+{w}+{} -> {out}", a.text, b.text));
        }
    }
    println!(
        "Glued zh+en+zh: {n} sentences, {} wrong ({:.1}%)",
        bad.len(),
        100.0 * bad.len() as f64 / n.max(1) as f64
    );
    for b in bad.iter().take(15) {
        println!("  wrong: {b}");
    }

    // English words and identifiers followed by a tone digit must stay as typed.
    let mut tokens: Vec<String> = [
        "user2024", "mp3", "mp4", "web3", "h264", "gpt4", "win7", "ipv4", "x86", "es6",
        "form4", "item3", "room4", "html5", "css3", "python3", "v1.3.4", "page3", "level4",
        "step6", "top7", "ios17", "m4a", "p2p", "b2b",
    ]
    .iter()
    .map(|t| t.to_string())
    .collect();
    for (i, w) in words_all.iter().skip(100).take(2900).enumerate() {
        tokens.push(format!("{w}{}", ['3', '4', '6', '7'][i % 4]));
    }
    let bad: Vec<String> = tokens
        .iter()
        .filter_map(|t| {
            let out = run(&mut engine, &format!("{t}<E>"));
            (out != *t).then(|| format!("{t}->{out}"))
        })
        .collect();
    println!(
        "English + digit: {} tokens, {} changed ({:.1}%)",
        tokens.len(),
        bad.len(),
        100.0 * bad.len() as f64 / tokens.len() as f64
    );
    for b in bad.iter().take(15) {
        println!("  wrong: {b}");
    }

    // English: common words typed lowercase and closed with a space.
    let words = eng.words_by_rank();
    for (lo, hi) in [(0usize, 300usize), (300, 1000), (1000, 3000), (3000, 10000)] {
        let slice = &words[lo.min(words.len())..hi.min(words.len())];
        let mut bad = Vec::new();
        for w in slice {
            let out = run(&mut engine, &format!("{w}_<E>"));
            if out != format!("{w} ") {
                bad.push(format!("{w}->{out}"));
            }
        }
        println!(
            "English ranks {lo}-{hi}: {} words, {} wrong ({:.1}%)",
            slice.len(),
            bad.len(),
            100.0 * bad.len() as f64 / slice.len().max(1) as f64
        );
        for b in bad.iter().take(25) {
            println!("  wrong: {b}");
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("play") => play(&args.next().unwrap_or_default()),
        Some("eval") => eval(),
        _ => {
            eprintln!("usage: keyflip-cli play <keys> | eval");
            std::process::exit(2);
        }
    }
}
