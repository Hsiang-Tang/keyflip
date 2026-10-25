# Specification: product demo film (005-demo-film)

Status: implemented 2026-10-25.

## Goal

A short film that shows what keyflip does while someone types: Zhuyin turning into
English mid-word, Shift-started words, identifiers that stay English, Tab completion,
a wrong guess fixed with Tab, and a flipped word remembered. It shows the product, not
measurements: no statistics on screen.

## Requirements

- DF-1 Every piece of composition text, segment kind, cursor, and commit on screen comes
  from the engine. The film replays a trace produced by `keyflip-cli trace`; no
  composition text is written by hand.
- DF-2 `keyflip-cli trace <keys>` prints one JSON object per key: the key (code, typed
  text, Shift), `handled`, `commit`, the preedit segments with their kind, the cursor,
  and the visible candidates. It runs the engine with an in-memory user model, so a word
  flipped with Tab earlier in the same run is remembered later in the run (UL-13). The
  model is never read from or written to disk (constitution principle 5).
- DF-3 Layout, 1920x1080 at 60 fps: the text being typed fills the upper half; a line
  drawing of the main block of a standard Zhuyin keyboard (number row to space bar, each
  key labelled with its letter and Zhuyin symbol) fills the lower half. A pressed key
  sinks and lights; Shift stays lit while held; Tab and Enter light like any key.
- DF-4 Segment styles: pending Zhuyin has a dotted underline, English (Latin) a soft
  accent underline, converted Chinese is plain ink, a suggestion is dimmed text after the
  cursor, committed text is plain ink with no underline.
- DF-5 Script: one continuous message typed by a person, in beats that each show one
  behavior. Beats: plain Zhuyin; a Shift-started word; an all-caps abbreviation;
  lowercase English that flips from Zhuyin mid-word; a path that keeps a short
  ambiguous word English; Tab completion; a wrong guess fixed with Tab; the same word
  typed again and kept English. Each beat may show one caption of at most four English
  words, small, outside the text area.
- DF-6 Motion: typing at a human rhythm (about 8 to 10 keys per second with small
  variation). The frame never moves: on the flip, completion, correction, and memory
  beats the line being typed grows in place (about 1.3x, from its left edge) while the
  other lines dim, so the keyboard and every pressed key stay fully in frame. No shake,
  rotation, or overshoot; a finished line holds at least 1.2 s before the next.
- DF-7 Outputs: an MP4 with a quiet score under clicky (blue-switch) key sounds from CC0
  recordings, a press click on key down and a softer release click on key up; a README GIF of the opening
  beats, at most 5 MB. Files over 10 MB are not committed (repository rule); the MP4 is
  kept locally and attached to a release on request.
- DF-8 QA: an automatic check every 0.1 s that text and captions stay inside a 64 px
  safe margin and do not overlap the keyboard, and that the keyboard is never cropped,
  plus a contact sheet of stills.
- DF-9 Reproducible: one command rebuilds the trace, frames, audio, MP4, and GIF from
  the script.

## Not doing

- Statistics, accuracy numbers, or comparison counters on screen.
- Real screen recordings; the keyboard and text are drawn.
- Narration.
