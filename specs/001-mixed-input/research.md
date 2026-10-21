# Survey of existing input methods

Everything read before starting, gathered so a new contributor does not have to redo it.
Survey date: 2026-10-21. Each project was shallow-cloned and only its source was read; none
were built or run. Items marked "unverified" were not confirmed in code by the surveyor.

## 1. Summary

- Approaches fall into three groups:
  - Modifier-key gestures (Shift tap, CapsLock): 0C, UnifyIME, vChewing, ruhi. Most side effects.
  - Automatic judgment from "can the key sequence form legal Zhuyin": shanjie, BearSpark,
    zhijeida, vChewing's mixed typing, TronIn.
  - Whole-sentence scoring: UnifyIME, TronIn.
- keyflip is mainly the second group plus a little scoring, and uses no modifier taps at all
  (reasons in section 11).
- The most important shared fact: when the user types tones, "the key sequence cannot be
  Zhuyin" is strong evidence of English. For people who do not type tones every approach
  works poorly (zhijeida and vChewing say so explicitly).
- No project truly solves two-letter abbreviations (`ai`, `jp`, `up`, `i`). Everyone defaults
  to Chinese and relies on Tab or the candidate list to switch back to English.

## 2. TronIn (macOS, closed source)

- The public repo `pct/tronin` holds only LICENSE, NOTICE, and README, no code. Everything
  below comes from release notes (v0.1.0 to v0.2.41), 27 closed issues, and the website
  manual, none verified against code.
- Judgment (inferred):
  - Every key is both Zhuyin and English; both readings are kept, the likelier one is shown
    and the other is second in the candidates.
  - Evidence: whether the Zhuyin forms characters and whether they are actually common
    (v0.2.22; older versions only asked whether any character could be formed, so `do` was
    judged 剋), a common English word list (wordfreq), brand and software names (over 5,000
    Wikidata entries), and identifier rules (`z-index`, `.json`, `v1` to `v9`, email, URL).
  - Hard structural rules: ㄒ ㄐ ㄑ can only precede ㄧ or ㄩ, so a digit after `v` cannot be
    Zhuyin (v0.2.24); ㄦ does not follow an initial, so "letter plus `-`" that forms no
    character must be English (v0.2.20).
  - Whole-sentence comparison with the McBopomofo lexicon and bigram segmentation; real words
    always beat fault tolerance (v0.2.12).
  - Rules apply only at commit time; each keystroke while typing is unchanged; it never
    outputs raw Zhuyin text (v0.2.11).
- Keys:
  - Holding Shift while composing forces English and keeps the Chinese before it; a mistaken
    press is undone by Backspace on that letter.
  - Right-Shift tap switches to pure English mode (can be turned off). This is the gesture it
    kept; keyflip does not adopt it.
  - The number row is Zhuyin and tones by default; Enter after digits outputs literal
    digits; the numpad always outputs digits.
  - Space: first tone if none, commit if a tone is present; Space is also a Chinese/English
    break point.
  - Tab: pressed mid-English, the word is committed and Tab is returned to the app (terminal
    completion).
  - Esc only closes the window, it does not clear the composition.
- Solved pain points (all pitfalls for keyflip to avoid):
  - Fast-typing tolerance (early tone, swapped medial/final) wrongly merged words and was
    fixed over five releases; several words became wrong characters. Conclusion: do not do
    tolerance.
  - English swallowed as Zhuyin: `debug` followed by Chinese, `asus`, `tw`, `cp`, `.exe`,
    `z-index`, `v3`, `http://`.
  - `-` and `.` after English became half-width, making some characters untypable (issue
    #16). Fix: Space is a word boundary and a focus change resets.
  - Learning pollution: imported mixed text was learned as untypable Zhuyin indexes.
  - Candidate window positioning: Chromium, Electron, Google Docs report empty coordinates,
    full-screen levels, dual screens.
  - CapsLock hijacks switching; a terminal app (Ghostty) eats characters.
  - The engine runs in a separate subprocess and a backup engine behaved differently from the
    main one, causing sudden "dumbness".
- Unsolved: two-letter abbreviations default to Chinese (#6); only the Dachen layout (#2);
  typing `---` in a note app is first judged Zhuyin (#25).
- Reusable: the McBopomofo lexicon (MIT), wordfreq (CC BY-SA). Design ideas: keep both
  readings, hard structural rules, Space as a hard boundary, sanity checks on candidate window
  coordinates.

## 3. UnifyIME (macOS, Swift)

- Approach: keep the whole raw key buffer and replay it from the start on each key, with a
  prefix cache of the latest 64 checkpoints.
- Debounce: when the buffer contains English letters, each key cancels the earlier schedule
  and delays re-judgment by 600 / 400 / 150 (default) / 100 / 80 ms; it is forced before
  commit or cursor movement (`RuntimeSupport.swift:121, 219-243`).
- Re-judgment: first decide whether the whole string is an exact English word or an
  extendable English prefix; with up to 3 keys and evidence for Chinese, keep Chinese.
  Otherwise do dynamic programming with at most 6 candidates per start; each key gap costs 5
  points. An exact English word scores about 200000 times its length; a tentative English
  prefix 180000 times its length.
- Syllable reordering (`BopomofoOrderCorrection.swift`): symbols are placed into four slots
  (initial, medial, final, tone) and the result must be a syllable in the lexicon. It uses no
  timing.
- Modifier tap (`ShiftLanguageGesture.swift`, 31 lines):
  - Needs flagsChanged events. The target key going down with no other modifier sets pending;
    on release, if pending still holds, it switches.
  - Any keyDown cancels pending; left and right Shift are told apart by keyCode.
  - **No time threshold**: a long press and release also counts as a tap.
- Editing during composition: the cursor is derived from one true cursor; manual locks and
  automatic decisions are stored separately; candidates keep their language and replacement
  range.
- Ranking: rule-based base score, neural residual limited to plus or minus 56 per candidate;
  not enabled by default.
- Unsolved: homophones, ambiguity between `system` and the next syllable, failures on long
  sentences; issue #4 ("cannot type digits without switching") and #5 ("fast-typing key order
  swapped") are still open.
- Reusable: whole-buffer replay with a prefix cache, separating manual locks from automatic
  decisions, debounce plus forcing before commit.

## 4. zhijeida (macOS, Swift; standard Dachen)

- Correction: it is standard Dachen layout, not Dayi.
- Assumes the user types tones, with Space for first tone. One Chinese unit has at most one
  syllable, so a string that is not a legal syllable is English.
- It is a table lookup: `bpmf.tsv` (key sequence to characters) generated from McBopomofo
  data. `split` order: a whole-string syllable is Chinese; all digits is English (but an
  all-digit legal syllable such as `284` or `18` is judged Chinese first); with non-letter
  characters it enumerates tail segments, finds the longest legal syllable tail, and treats
  the prefix as English.
- A run of glued letters (`sdkep`) is not split; default English, fixed via candidates.
- Punctuation: `, . ; / -` are also finals, so after Chinese they are held; if the next key is
  a tone key forming a legal syllable they are Zhuyin (`-6` is 而), otherwise they settle as
  punctuation.
- Space: settles pending, and the English tail keeps one space. Enter with the candidate
  window open selects, otherwise commits; Backspace deletes the last pending key first.
- Candidates: multi-character words first, single characters after, the raw keys inserted
  between them; picked words gain 3.0 points, capped at 9.0.
- IMK pitfalls (README): overriding `handle` makes IMK stop calling `inputText`; returning
  false in `inputText` clears the composition; the IMK process cannot get modifier keys.
- Unsolved: does not suit users who skip tones; `up` and `i` are indistinguishable; unigram
  only.

## 5. BearSpark (C++, OpenVanilla)

- Judgment: every key is first fed as Zhuyin into the reading buffer, then the reading is
  mapped back through the keyboard layout to a key sequence and compared with the actual
  input; on mismatch the whole string becomes ASCII. No English dictionary needed
  (`OVIMSmartMandarin.cpp:342-361`).
- Supports only standard, Eten, and IBM layouts; layouts that reuse a key across fields (such
  as Hsu) are unsupported.
- Shift plus letter goes straight to ASCII. A tone key finishes the reading: in the lexicon
  it becomes Zhuyin, otherwise ASCII.
- Space: an ASCII string gets one space appended; Enter commits the raw letters without a
  space.
- Tab: cycles among the typed letters, English, and Zhuyin.
- Digits: 3 4 6 7 with no preceding initial, medial, or final are digits; digits followed by
  Zhuyin split as `3樓`; Space accepts only readings of two keys or more, so `5g` stays as is.
- Handled: `g` plus Enter commits `g` rather than 師; a bare initial is not treated as a word.

## 6. shanjie (Rust core plus macOS shell)

- Shadow mechanism: beside an unfinished syllable it keeps the raw key caps (up to 32) and
  counts same-slot overwrites. It fires only at the moment of Space, when the shadow is a
  complete word of the word table and the overwrite count is enough.
- The English window lists as-typed, capitalized, and all-caps forms; after choosing, the word
  stays in the composition.
- When the shadow is at least 3 letters and is a prefix of some word, letters are shown
  directly while typing.
- Shift plus letter starts or continues an English segment; when the letters are already a
  whole word and the next Zhuyin key joins no word, it fixes automatically.
- Space: the first one ends the English segment (swallowed), the second inserts a space.
- Digits: with an empty buffer, 3 4 6 7 are always digits; inside a string, they are scored
  by conditional probability on the preceding context with a threshold of 3.0; scoring
  segmentation accuracy 89.8%, structural rules 85.2% (unverified).
- Measurements: messages containing English tokens are 22.8%; word-table recall 99.48%;
  with 2% simulated typos, false triggers stay under 2 per thousand syllables.
- Borrowable: measure the need before deciding to build; offer candidates only before
  auto-committing and relax gradually; the simulator reports triggers from the engine instead
  of re-implementing the judgment in the tool; generate the word table at build time and
  check its SHA-256.

## 7. vChewing (macOS)

- Mixed-typing fallback (off by default, since v4.4.1): when the buffer cannot become one
  reading by Zhuyin key order, it is treated as English intent
  (`Typewriter_MixedAlphanumerical.swift`). ASCII typed with Shift held never counts as a
  Zhuyin key; Shift+Space commits the whole buffer as raw text plus a half-width space.
- auto-split: `hello` plus Chinese splits an ASCII prefix from a Zhuyin suffix. A suffix of 3
  or more keys prefers splitting; a short suffix prefers whole-string Zhuyin. A pure digit
  suffix is not split, so `192.168.100.1` is not cut into Zhuyin.
- Latch: once judged English, every later printable ASCII is committed immediately; Enter,
  Backspace, Delete, and Esc release it.
- Shift tap detection (`ShiftKeyUpChecker.swift`): uses only IMK events; any keyDown cancels;
  holding over 0.2 s does not count; a 0.05 s cooldown after firing prevents Electron from
  splitting one release into two rounds.
- CapsLock tap: only flagsChanged with keyCode 57, throttled to 0.08 s.
- **Known problems (the main reason keyflip uses no modifier gestures)**:
  - A Shift tap cannot be detected while the CapsLock light is on (vChewing says so in its
    own preferences text).
  - CapsLock switching delay: USB hubs, Touch Bar, IMK main-thread stalls; the official advice
    is `hidutil` with `CapsLockDelayOverride` (unverified).
  - After switching apps flagsChanged is not received, Shift taps stop working until CapsLock
    is pressed (#618 still open).
  - Shift taps are judged twice in Discord (Electron) (#460, #568); remote English mode in
    Jump Desktop turns full-width (#596); JetBrains and Rayon cannot type lowercase when "use
    CapsLock to switch ABC" is off.
  - The first key after switching back to the input method occasionally does not compose.
- Fixed: `e` becoming ㄍ after `apple` plus Space (#614); two commits from a double Space;
  first-tone Space fighting with the Shift+Space escape hatch; short English `ls` swallowed.
- Trade-offs: leading tones, tone overwrite, and automatic reading correction are all off.
- Reusable: judge by "can a reading be formed by key order"; score English-intent evidence
  and Zhuyin evidence separately; make the latch an independent flag with an explicit release
  key.

## 8. KnowType (macOS, Rime pinyin)

- Uses no Shift or CapsLock: Option+`/` toggles Chinese and ASCII, Option+`.` toggles
  punctuation, Shift+Space toggles full/half width. It registers only keyDown.
- Code-like token protection (`TextProtection.swift`), checked in order: protected apps
  (Terminal, Xcode, iTerm), URL, email, path, command (subcommand whitelist plus operators),
  code shape (`;`, `{`, `=>`, `let`, `func`).
- Protected ranges are output as a range list, not a boolean.
- Weakness: lots of regex plus whitelists; in #227 a bare-domain regex treated `user.name` as a
  URL.
- It follows the pinyin path; its trade-offs do not necessarily apply to Zhuyin.

## 9. ruhi (Windows TSF, Rust)

- The engine is linked into a single DLL as a library, with no separate process.
  `OnTestKeyDown` and `OnKeyDown` each recompute modifier state. Shift uses `GetKeyState` plus
  `GetAsyncKeyState`; CapsLock reads the toggle state.
- The candidate window is a self-drawn Direct2D `WS_EX_NOACTIVATE|TOOLWINDOW|TOPMOST` window,
  with inline and floating display modes.
- Identifier classification (`western/fsm.rs:41-93`): all digits and dots is Number;
  containing `_` and all caps is ScreamingSnake, else Snake; only alphanumerics and `-` is
  Kebab; lowercase followed by uppercase inside is Pascal or Camel; first letter capital and
  the rest lowercase is Title; all caps is Upper; the rest is Lower. Identifiers are always
  protected from splitting.
- The design blueprint enables automatic mixed typing only on the 41-key Dachen layout; Hsu
  and pinyin rely on CapsLock or Shift.
- Windows pitfalls (`docs/TSF_WIN32_COMPATIBILITY_AND_INSTALLER_SPEC.md`):
  - Statically importing `TaskDialogIndirect` made conhost and LibreOffice fail to load; at
    release the import table is audited with pefile and must not contain `comctl32`.
  - `RegisterProfile` with a non-zero `hklsubstitute` flashes Win32 windows back to English;
    it must be NULL.
  - Cursor coordinates in LibreOffice and Photoshop are unreliable, so it falls back to
    inline.
  - After committing in Notepad you must Collapse and SetSelection, or the next character
    overwrites the committed one.
- Install (Inno Setup): `taskkill ctfmon`, `regsvr32`, `icacls` granting ALL_APPLICATION_PACKAGES,
  restart ctfmon. There is no automated test at the TSF layer.
- Unverified: latency, Chrome, Office, password fields, elevated windows.

## 10. 0C (Go engine, Rust TSF thin shell, macOS Swift)

- The architecture closest to keyflip: TSF is a thin shell that talks to the Go engine over a
  named pipe; a 4-byte little-endian length plus JSON; requests are semantic key events and
  responses are always a full snapshot (Handled, Commit, Preedit with styles, Cursor in
  UTF-16, Candidates). The shell keeps no state.
- Every I/O has a 1-second timeout; after a disconnect it reconnects, reopens the session,
  and replays the requests of the current composition.
- Key rules: when the engine is unavailable keys are not consumed (once `OnTestKeyDown` says
  consume, hosts such as WPF stop handling the key themselves); combos with Ctrl, Alt, or Win
  commit the composition first and then return the key to the host.
- A Shift tap toggles (holding over 0.3 s does not count); mixed typing is off by default
  (`mixed.go`, 893 lines), keeping the physical keys in `pendingKeys{raw, accepted, choice}`
  until a tone or boundary decides.
- Windows pitfalls:
  - CreateProcess is impossible inside the AppContainer (search box) sandbox, so the engine
    stays resident from login; the pipe ACL is opened to ALL APPLICATION PACKAGES; the
    candidate window is created inside the host process.
  - Password fields check the compartment, read-only, and InputScope, send no keys and record
    nothing; a field marked only `IS_PRIVATE` composes normally but the engine does not learn.
  - Games: UI-less needs `UIELEMENTENABLED` registered; COM-less (Minecraft) needs `COMLESS`.
  - 32-bit apps need an i686 DLL; Chromium and WebView2 layout is asynchronous; when
    `GetTextExt` cannot get a position, register `ITfTextLayoutSink` to resend.
  - The answer of `OnTestKeyDown` must match `OnKeyDown`.
- Install: NSIS with signing; fixed path `%LOCALAPPDATA%\Zuin\tsf\0c_tsf.dll`; file swaps use
  `.new` and `.old` renames plus a hash check.
- Tests: many Go key-scenario tests; Rust integration tests run against a real server; about
  40 interactive PowerShell acceptance scripts; CI runs on ubuntu only.
- Unverified: elevated windows; the P95 latency of 7 to 16 ms is the repository's own claim.

## 11. Design implications for keyflip

- No modifier tap of any kind: vChewing's Shift and CapsLock taps have a long list of
  unsolved problems (delay, Electron double firing, events lost after app switch, failure
  with the CapsLock light on). keyflip receives keyDown only, so such events never arrive.
- Judgment core: whether the key sequence can become legal Zhuyin (the common point of
  vChewing, BearSpark, zhijeida, TronIn), with the lexicon's syllable-prefix set expressing
  the hard structural rules.
- No fast-typing tolerance (TronIn spent five releases fixing wrong merges).
- Architecture follows 0C: thin shell, named pipe, full snapshot; the shell does not judge.
- On Windows avoid two of ruhi's pitfalls: NULL for `hklsubstitute` and no `comctl32` import;
  keep `OnTestKeyDown` and `OnKeyDown` consistent; collapse the selection after committing.
- Identifier protection (ruhi's classification, KnowType's range list) is later work; see
  `docs/ROADMAP.md`.
- Tab as the flip key (a simplified version of BearSpark's three-state cycle); Enter commits
  raw letters (BearSpark and TronIn agree).
- Digits: with nothing in the buffer, a digit is a digit (shanjie, BearSpark, TronIn agree);
  a digit run keeps taking digits.

## 12. Not read, or only skimmed

- TronIn's actual algorithm has no source to read.
- shanjie's `digits.md` implementation and the other parts of `PLAN.md` were not checked line
  by line.
- No mixed-typing unit tests were found for BearSpark.
- All real-world performance and compatibility numbers are the authors' own claims.
