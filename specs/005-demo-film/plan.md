# Plan: product demo film (005-demo-film)

## Pipeline

```
demo/script.json ──> keyflip-cli trace ──> demo/out/trace.json
                                              │
demo/scene.html (+ scene.js) <────────────────┘   page state = f(time)
        │
demo/render.mjs ── headless Chrome, 1/60 s per frame, screenshots ──> ffmpeg ──> MP4, GIF
demo/music.py  ── synthesized score and key clicks from the key times ──┘
```

- Same technique as the maintainer's earlier films (anatomy, Alphalia): the page exposes
  `window.film.seek(t)`, the renderer advances time frame by frame. No video framework
  dependency; `playwright-core` drives the installed Google Chrome.
- `script.json` lists beats. Each beat has keys in the acceptance notation, a typing
  rhythm, an optional caption, and an optional focus (the typed line grows in place). `render.mjs` expands it into
  timed key events with a seeded random jitter, calls `keyflip-cli trace` once for the
  whole message (so learning carries across beats), and writes `out/timeline.json`.
- The keyboard is drawn in SVG from a layout table (rows, key widths, letter, Zhuyin
  label) matching the Apple Zhuyin keyboard. Ink lines on paper, one orange accent,
  the visual language of the network-switch-anatomy film.
- Committed text lands in a message area above the live composition; the composition
  shows the engine's preedit with DF-4 styles and a caret at the reported cursor.
- Audio: `music.py` writes a soft synthesized pad under blue-switch key sounds cut from CC0
  Freesound recordings (`demo/sounds/`, sources in `SOURCES.md`): a press on key down, a
  release on key up, small pitch and level variation; standard library only.

## Where things go

- `demo/`: script, scene, renderer, music, `package.json`; `demo/out/` ignored.
- `demo/keyflip-demo.gif`: README GIF (committed, under 5 MB).
- The MP4 stays in `demo/out/` and is copied to the maintainer's media folder.
