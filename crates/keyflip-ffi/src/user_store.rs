//! User files (004-user-lexicon). `phrases.txt` is written by the user;
//! `learned.tsv`, `english.tsv`, and `short-english.tsv` are written here. The engine only sees the
//! in-memory model (UL-10).

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use keyflip_core::user::{UserModel, PHRASES_HEADER};

pub const PHRASES: &str = "phrases.txt";
pub const LEARNED: &str = "learned.tsv";
pub const ENGLISH: &str = "english.tsv";
pub const SHORT: &str = "short-english.tsv";

/// Files keyflip writes: name, loader (returns malformed lines), serializer.
type Load = fn(&mut UserModel, &str) -> usize;
type Dump = fn(&UserModel) -> String;
const WRITTEN: [(&str, Load, Dump); 3] = [
    (LEARNED, UserModel::load_learned, UserModel::learned_tsv),
    (ENGLISH, UserModel::load_english, UserModel::english_tsv),
    (SHORT, UserModel::load_short, UserModel::short_tsv),
];

pub struct UserStore {
    pub model: Arc<Mutex<UserModel>>,
    dir: PathBuf,
    /// Model generation as of the last load or save.
    saved: u64,
    times: HashMap<&'static str, Option<SystemTime>>,
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Invalid UTF-8 is replaced, so those lines count as malformed.
fn read_text(path: &Path) -> String {
    fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

fn write_atomic(path: &Path, text: &str) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

fn copy_beside(path: &Path, suffix: &str) {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    let _ = fs::copy(path, PathBuf::from(name));
}

impl UserStore {
    /// Failures leave an empty model: typing never depends on the files.
    pub fn open(dir: &Path) -> UserStore {
        let _ = fs::create_dir_all(dir);
        let phrases = dir.join(PHRASES);
        if !phrases.exists() {
            let _ = write_atomic(&phrases, PHRASES_HEADER);
        }
        let mut store = UserStore {
            model: Arc::new(Mutex::new(UserModel::default())),
            dir: dir.to_owned(),
            saved: 0,
            times: HashMap::new(),
        };
        store.reload();
        store
    }

    fn lock(&self) -> MutexGuard<'_, UserModel> {
        self.model.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn generation(&self) -> u64 {
        self.lock().generation()
    }

    /// Returns true when `name` changed on disk since it was last read or written.
    fn changed(&mut self, name: &'static str) -> bool {
        let time = modified(&self.dir.join(name));
        self.times.insert(name, time) != Some(time)
    }

    /// Reloads files changed on disk (UL-9). A written file with malformed
    /// lines is copied to `.corrupt` and rewritten clean; the phrases file is
    /// never rewritten.
    pub fn reload(&mut self) {
        if self.changed(PHRASES) {
            let text = read_text(&self.dir.join(PHRASES));
            self.lock().load_phrases(&text);
        }
        let mut rewrite = false;
        for (name, load, _) in WRITTEN {
            if self.changed(name) {
                let path = self.dir.join(name);
                if load(&mut self.lock(), &read_text(&path)) > 0 {
                    copy_beside(&path, ".corrupt");
                    rewrite = true;
                }
            }
        }
        self.saved = self.generation();
        if rewrite {
            self.save();
        }
    }

    pub fn save_if_changed(&mut self) {
        if self.generation() != self.saved {
            self.save();
        }
    }

    fn save(&mut self) {
        for (name, _, dump) in WRITTEN {
            let text = dump(&self.lock());
            let path = self.dir.join(name);
            if write_atomic(&path, &text).is_ok() {
                self.times.insert(name, modified(&path));
            }
        }
        self.saved = self.generation();
    }

    /// Forgets everything learned (UL-6), keeping `.bak` copies so a clear by
    /// mistake can be undone by hand.
    pub fn clear(&mut self) {
        for (name, _, _) in WRITTEN {
            let path = self.dir.join(name);
            if path.exists() {
                copy_beside(&path, ".bak");
            }
        }
        self.lock().clear_learned();
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("keyflip-test-{tag}-{}-{nanos}", std::process::id()))
    }

    fn rs(s: &str) -> Vec<String> {
        s.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn first_open_creates_the_template_and_learning_is_saved() {
        let dir = temp_dir("open");
        let mut store = UserStore::open(&dir);
        let template = fs::read_to_string(dir.join(PHRASES)).unwrap();
        assert!(template.starts_with("# keyflip user phrases"));
        assert!(!dir.join(LEARNED).exists(), "nothing learned, nothing written");

        store.model.lock().unwrap().learn(&rs("ㄉㄜ˙"), "地", "ㄇㄢˋ");
        store.save_if_changed();
        let saved = fs::read_to_string(dir.join(LEARNED)).unwrap();
        assert!(saved.contains("ㄉㄜ˙\t地\tㄇㄢˋ\t1\t1"), "{saved}");
        assert!(!dir.join("learned.tmp").exists());

        let again = UserStore::open(&dir);
        assert_eq!(again.model.lock().unwrap().learned_len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_corrupt_learned_file_is_kept_aside_and_rewritten() {
        let dir = temp_dir("corrupt");
        fs::create_dir_all(&dir).unwrap();
        let mut bytes = "ㄉㄜ˙\t地\tㄇㄢˋ\t2\t5\nhalf a li".as_bytes().to_vec();
        bytes.extend([0xff, 0xfe, b'\n']);
        fs::write(dir.join(LEARNED), &bytes).unwrap();

        let store = UserStore::open(&dir);
        assert_eq!(store.model.lock().unwrap().learned_len(), 1);
        assert_eq!(fs::read(dir.join("learned.tsv.corrupt")).unwrap(), bytes);
        let clean = fs::read_to_string(dir.join(LEARNED)).unwrap();
        assert!(!clean.contains("half"), "{clean}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hand_edits_are_picked_up_on_reload_and_clear_forgets_everything() {
        let dir = temp_dir("reload");
        let mut store = UserStore::open(&dir);
        // Modification times can be coarse; make the edit visibly newer.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        fs::write(dir.join(PHRASES), "鍵翻\tㄐㄧㄢˋ ㄈㄢ\n").unwrap();
        store.reload();
        assert_eq!(store.model.lock().unwrap().phrases_from("ㄐㄧㄢˋ").len(), 1);
        assert_eq!(fs::read_to_string(dir.join(PHRASES)).unwrap(), "鍵翻\tㄐㄧㄢˋ ㄈㄢ\n");

        store.model.lock().unwrap().learn(&rs("ㄉㄜ˙"), "地", "");
        store.model.lock().unwrap().learn_word("docker");
        store.model.lock().unwrap().learn_short("ai");
        store.save_if_changed();
        assert!(fs::read_to_string(dir.join(ENGLISH)).unwrap().contains("docker\t1\t"));
        assert!(fs::read_to_string(dir.join(SHORT)).unwrap().contains("ai\t"));
        store.clear();
        assert_eq!(store.model.lock().unwrap().learned_len(), 0);
        let saved = fs::read_to_string(dir.join(LEARNED)).unwrap();
        assert!(!saved.contains('地'));
        let backup = fs::read_to_string(dir.join("learned.tsv.bak")).unwrap();
        assert!(dir.join("english.tsv.bak").exists());
        assert!(dir.join("short-english.tsv.bak").exists());
        assert_eq!(store.model.lock().unwrap().short_len(), 0);
        assert!(backup.contains('地'), "a clear can be undone by hand");
        let _ = fs::remove_dir_all(&dir);
    }
}
