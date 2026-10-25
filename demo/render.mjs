// Demo film director (specs/005-demo-film).
// usage: node demo/render.mjs timeline|stills|qa|video|gif|all
// timeline: expand script.json into timed keys and replay them through `keyflip-cli trace`.
// The page draws a frame as a function of time; this script seeks it 1/fps s per frame,
// screenshots it, and ffmpeg encodes the frames with the score from music.py.
import { createRequire } from "module";
import fs from "fs";
import path from "path";
import url from "url";
import { execFileSync } from "child_process";

const { chromium } = createRequire(import.meta.url)("playwright-core");
const here = path.dirname(url.fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const out = path.join(here, "out");
const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const FFMPEG = "ffmpeg";
const mode = process.argv[2] || "all";
fs.mkdirSync(out, { recursive: true });

// Seeded random numbers so every rebuild types with the same rhythm.
function rng(seed) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let x = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    x = (x + Math.imul(x ^ (x >>> 7), 61 | x)) ^ x;
    return ((x ^ (x >>> 14)) >>> 0) / 4294967296;
  };
}

const tokens = (keys) => keys.match(/<[^>]+>|./gu) || [];
const ROW = {};
["`1234567890-=", "qwertyuiop[]\\", "asdfghjkl;'", "zxcvbnm,./"].forEach((r, i) => [...r].forEach((c) => (ROW[c] = i)));
const keyId = (code) => (code.length === 1 ? code : code === "Backspace" ? "Backspace" : code);

// Segments that changed into English, or anything Tab changed, flash once (DF-4).
function flashes(prev, cur, code) {
  const out = [];
  cur.forEach((seg, i) => {
    const before = prev[i];
    const changed = before && (before.kind !== seg.kind || before.text !== seg.text);
    if ((seg.kind === "Latin" && changed) || (code === "Tab" && seg.kind !== "Suggestion" && (changed || !before))) out.push(i);
  });
  return out;
}

function buildTimeline() {
  const script = JSON.parse(fs.readFileSync(path.join(here, "script.json"), "utf8"));
  const keys = script.beats.map((b) => b.keys).join("");
  const cli = path.join(root, "target/release/keyflip-cli");
  execFileSync("cargo", ["build", "-q", "--release", "-p", "keyflip-cli"], { cwd: root, stdio: "inherit" });
  const trace = execFileSync(cli, ["trace", keys], { encoding: "utf8" }).trim().split("\n").map((l) => JSON.parse(l));
  const total = script.beats.reduce((n, b) => n + tokens(b.keys).length, 0);
  if (trace.length !== total) throw new Error(`trace has ${trace.length} events, script has ${total} keys`);

  const rand = rng(20261025);
  const events = [], presses = [], music = [], captions = [], pushes = [], beatStarts = [];
  let t = script.intro, k = 0, prev = [];
  script.beats.forEach((beat, bi) => {
    t += beat.pause ?? (bi === 0 ? 0 : 0.12);
    beatStarts.push(t);
    for (const tok of tokens(beat.keys)) {
      const e = trace[k++];
      const step = beat.slow ?? 0.1 + rand() * 0.06;
      if (e.shift) {
        t += 0.05;
        presses.push({ id: "Shift", down: t - 0.07, up: t + 0.08 });
        music.push({ t: t - 0.07, up: t + 0.08, code: "Shift", row: 3 });
      }
      events.push({ ...e, t, flash: flashes(prev, e.preedit, e.code) });
      presses.push({ id: keyId(e.code), down: t, up: t + 0.09 });
      music.push({ t, up: t + 0.09, code: e.code, row: ROW[e.code] ?? 4 });
      prev = e.preedit;
      t += step;
      if (tok === "<E>" && !e.handled) t += 0.55;
      if (tok === "_") t += 0.04;
    }
    t += beat.hold ?? 0;
  });
  beatStarts.push(t);

  // Merge overlapping Shift presses so a held Shift stays lit (PR).
  const shift = presses.filter((p) => p.id === "Shift").sort((a, b) => a.down - b.down);
  const merged = [];
  for (const p of shift) {
    const last = merged[merged.length - 1];
    if (last && p.down <= last.up + 0.04) last.up = Math.max(last.up, p.up); else merged.push({ ...p });
  }
  const allPresses = presses.filter((p) => p.id !== "Shift").concat(merged);

  script.beats.forEach((beat, bi) => {
    if (beat.caption) {
      const next = script.beats.findIndex((b, j) => j > bi && b.caption);
      const end = Math.min(beatStarts[bi] + 3.4, next < 0 ? Infinity : beatStarts[next] - 0.35);
      captions.push({ t: beatStarts[bi], end, text: beat.caption });
    }
    if (beat.push) {
      const until = beatStarts[Math.min(bi + (beat.pushBeats ?? 1), beatStarts.length - 1)];
      pushes.push({ t: beatStarts[bi] - 0.35, end: until + 0.25, scale: beat.push });
    }
  });

  const endCard = t + 1.3;
  const timeline = {
    fps: script.fps, duration: +(endCard + script.outro).toFixed(3), endCard,
    gifEnd: beatStarts[5] - 0.2,
    events, presses: allPresses, captions, pushes,
    keys: music.sort((a, b) => a.t - b.t),
    beats: script.beats.map((b, i) => ({ t: beatStarts[i], caption: b.caption ?? null })),
  };
  fs.writeFileSync(path.join(out, "timeline.json"), JSON.stringify(timeline));
  const text = events.map((e) => e.commit + (e.code === "Enter" && !e.handled ? "\n" : "")).join("");
  console.log(`timeline: ${events.length} keys, ${timeline.duration} s\n${text}`);
  return timeline;
}

async function withPage(timeline, fn) {
  const browser = await chromium.launch({ executablePath: CHROME, args: ["--force-color-profile=srgb"] });
  try {
    const page = await browser.newPage({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
    await page.goto(url.pathToFileURL(path.join(here, "scene.html")).href);
    await page.evaluate((tl) => window.film.load(tl), timeline);
    await page.evaluate(() => document.fonts.ready);
    return await fn(page);
  } finally {
    await browser.close();
  }
}

const seek = (page, t) => page.evaluate((x) => window.film.seek(x), t);

async function qa(timeline) {
  const problems = await withPage(timeline, async (page) => {
    const found = [];
    for (let t = 0; t < timeline.endCard; t += 0.1) {
      await seek(page, t);
      for (const p of await page.evaluate(() => window.film.qa())) found.push(`${t.toFixed(1)} s: ${p}`);
    }
    return found;
  });
  console.log(problems.length ? problems.join("\n") : "qa: OK");
  if (problems.length) process.exitCode = 1;
}

async function stills(timeline) {
  const dir = path.join(out, "stills");
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  const times = timeline.beats.map((b) => b.t + 0.9).concat(timeline.pushes.map((p) => p.end - 0.3), [timeline.endCard + 1.5]);
  await withPage(timeline, async (page) => {
    for (const [i, t] of times.sort((a, b) => a - b).entries()) {
      await seek(page, t);
      await page.screenshot({ path: path.join(dir, `s${String(i).padStart(2, "0")}.png`) });
    }
  });
  execFileSync(FFMPEG, ["-y", "-loglevel", "error", "-pattern_type", "glob", "-i", path.join(dir, "s*.png"),
    "-vf", "scale=640:-1,tile=4x6:padding=6:color=white", "-frames:v", "1", path.join(out, "contact.png")]);
  console.log(`stills: ${times.length} -> out/contact.png`);
}

async function video(timeline) {
  const dir = path.join(out, "frames");
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  const n = Math.round(timeline.duration * timeline.fps);
  await withPage(timeline, async (page) => {
    for (let f = 0; f < n; f++) {
      await seek(page, f / timeline.fps);
      await page.screenshot({ path: path.join(dir, `f${String(f).padStart(5, "0")}.jpg`), type: "jpeg", quality: 94 });
      if (f % 300 === 0) console.log(`frame ${f}/${n}`);
    }
  });
  const silent = path.join(out, "silent.mp4"), score = path.join(out, "score.wav");
  execFileSync(FFMPEG, ["-y", "-loglevel", "error", "-framerate", String(timeline.fps), "-i", path.join(dir, "f%05d.jpg"),
    "-c:v", "libx264", "-preset", "slow", "-crf", "18", "-pix_fmt", "yuv420p", silent]);
  execFileSync("python3", ["-I", path.join(here, "music.py"), path.join(out, "timeline.json"), score], { stdio: "inherit" });
  execFileSync(FFMPEG, ["-y", "-loglevel", "error", "-i", silent, "-i", score, "-c:v", "copy", "-c:a", "aac", "-b:a", "192k",
    "-shortest", "-movflags", "+faststart", path.join(out, "keyflip-demo.mp4")]);
  console.log("video: out/keyflip-demo.mp4");
}

function gif(timeline) {
  const silent = path.join(out, "silent.mp4"), palette = path.join(out, "palette.png"), gifOut = path.join(here, "keyflip-demo.gif");
  const vf = "fps=20,scale=960:-1:flags=lanczos";
  execFileSync(FFMPEG, ["-y", "-loglevel", "error", "-t", String(timeline.gifEnd), "-i", silent, "-vf", `${vf},palettegen=max_colors=64:stats_mode=diff`, palette]);
  execFileSync(FFMPEG, ["-y", "-loglevel", "error", "-t", String(timeline.gifEnd), "-i", silent, "-i", palette,
    "-lavfi", `${vf}[x];[x][1:v]paletteuse=dither=none:diff_mode=rectangle`, gifOut]);
  console.log(`gif: ${(fs.statSync(gifOut).size / 1e6).toFixed(2)} MB, ${timeline.gifEnd.toFixed(1)} s`);
}

const timeline = mode === "timeline" || mode === "all" || !fs.existsSync(path.join(out, "timeline.json"))
  ? buildTimeline() : JSON.parse(fs.readFileSync(path.join(out, "timeline.json"), "utf8"));
if (mode === "qa" || mode === "all") await qa(timeline);
if (mode === "stills" || mode === "all") await stills(timeline);
if (mode === "video" || mode === "all") await video(timeline);
if (mode === "gif" || mode === "all") gif(timeline);
