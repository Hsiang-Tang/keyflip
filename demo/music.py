"""Score and key clicks for the demo film, synthesised from scratch (no samples, no third-party music).
usage: python3 -I demo/music.py TIMELINE.json OUT.wav
A calm D-major pad (Dmaj9 / Bm7 / Gmaj7 / A6sus, one chord per 2 bars at 94 bpm), a sparse electric-piano
motif with a soft swell at each beat start, and one quiet key click per key down from the timeline.
Standard library only, so it runs wherever the renderer runs."""
import array, json, math, random, struct, sys, wave

SR = 48000
timeline = json.load(open(sys.argv[1], encoding="utf-8")); out = sys.argv[2]
DUR = float(timeline["duration"]); N = int(round(SR * DUR))
L = array.array("d", bytes(8 * N)); R = array.array("d", bytes(8 * N))
BPM = 94; BEAT = 60 / BPM; BAR = 4 * BEAT
TAU = 2 * math.pi
rng = random.Random(25)
def hz(m): return 440.0 * 2 ** ((m - 69) / 12)
CH = [[50, 57, 61, 64, 66], [47, 54, 57, 62, 66], [43, 50, 54, 59, 62], [45, 52, 57, 59, 66]]  # Dmaj9 Bm7 Gmaj7 A6sus

def add(start, samples, gl, gr):
    i0 = int(start * SR)
    for k, v in enumerate(samples):
        i = i0 + k
        if i >= N: break
        if i >= 0: L[i] += v * gl; R[i] += v * gr

def pad(m, d, det):
    # two detuned sines and a soft octave, slow attack and release, gentle tremolo
    f1, f2 = TAU * hz(m) * (1 + det) / SR, TAU * hz(m) * (1 - det) / SR
    n = int(d * SR); a, r = 2.0 * SR, 2.5 * SR; o = [0.0] * n
    for k in range(n):
        e = min(1.0, k / a, (n - k) / r)
        o[k] = (math.sin(f1 * k) + math.sin(f2 * k) + 0.25 * math.sin(2 * f1 * k)) * e * (0.88 + 0.12 * math.sin(TAU * 0.09 * k / SR))
    return o

def epiano(m, d=2.2):
    # FM-ish bell: sine carrier with a decaying modulator, fast attack, long soft decay
    f = TAU * hz(m) / SR; n = int(d * SR); o = [0.0] * n
    for k in range(n):
        t = k / SR
        mod = 1.4 * math.exp(-t * 5) * math.sin(f * 2 * k)
        o[k] = (math.sin(f * k + mod) + 0.2 * math.sin(f * 3 * k) * math.exp(-t * 8)) * math.exp(-t * 2.2) * min(1.0, t / 0.003)
    return o

DEEP = {"Space", "Enter", "Shift", "Tab", "Backspace"}
def click(code, row):
    # a soft key tick: low-passed noise burst plus a small body thump; bigger keys sit lower
    deep = code in DEEP
    d = (0.038 if deep else 0.03) * (0.9 + 0.2 * rng.random()); n = int(d * SR)
    body = TAU * ((120 if deep else 190) + 18 * (4 - row) + rng.uniform(-12, 12)) / SR
    lp = 0.18 if deep else 0.3 + 0.03 * (4 - row)  # one-pole low-pass coefficient: darker for big keys
    y = 0.0; o = [0.0] * n
    for k in range(n):
        t = k / SR
        y += lp * ((rng.random() * 2 - 1) - y)
        o[k] = (0.9 * y * math.exp(-t * 170) + 0.55 * math.sin(body * k) * math.exp(-t * 95)) * min(1.0, t / 0.0006)
    return o

# pad, one chord per two bars, fading in over the first second via the master envelope
bars = int(DUR / BAR) + 2
for b in range(0, bars, 2):
    c = CH[(b // 2) % 4]
    for j, m in enumerate(c):
        pan = 0.5 + 0.3 * math.sin(j * 1.9)
        add(b * BAR - 0.4, pad(m, 2 * BAR + 2.5, 0.0011 * (j + 1)), 0.022 * (1.2 - pan), 0.022 * (0.2 + pan))

# sparse motif: two or three notes per bar, chord tones an octave up
MOTIF = [(0, 4), (1.5, 2), (3, 3)]
for b in range(bars):
    c = CH[(b // 2) % 4]; t0 = b * BAR
    if t0 < 1.5 or t0 > DUR - 2.5: continue
    for q, (beat, idx) in enumerate(MOTIF):
        if b % 2 and q == 2: continue
        pan = 0.3 + 0.4 * ((b + q) % 3) / 2
        add(t0 + beat * BEAT, epiano(c[idx] + 12), 0.032 * (1 - pan), 0.032 * pan)

# a soft swell at each beat start: the current chord root and fifth, one octave up
for beat in timeline.get("beats", []):
    t = float(beat["t"])
    if t < 0.5 or t > DUR - 2: continue
    c = CH[(int(t / BAR) // 2) % 4]
    for m in (c[0] + 12, c[1] + 12):
        add(t, pad(m, 3.0, 0.002), 0.02, 0.02)

# key clicks, panned slightly by key position so the board has width
for key in timeline.get("keys", []):
    code = str(key.get("code", "")); row = int(key.get("row", 2))
    x = (ord(code[0].lower()) % 13) / 12 if len(code) == 1 else 0.5
    pan = 0.4 + 0.2 * x
    add(float(key["t"]), click(code, row), 0.32 * (1 - pan) * 2, 0.32 * pan * 2)

# master: 1 s fade in, 2 s fade out, normalise to -3 dBFS
fi, fo = SR * 1.0, SR * 2.0
for i in range(N):
    g = min(1.0, i / fi, (N - i) / fo)
    L[i] *= g; R[i] *= g
peak = max(max(map(abs, L)), max(map(abs, R))) or 1.0
gain = 10 ** (-3 / 20) / peak
with wave.open(out, "wb") as w:
    w.setnchannels(2); w.setsampwidth(2); w.setframerate(SR)
    frames = bytearray()
    for i in range(N):
        frames += struct.pack("<hh", int(L[i] * gain * 32767), int(R[i] * gain * 32767))
    w.writeframes(bytes(frames))
print(f"wrote {out} ({DUR:.2f} s, {N} frames, gain {gain:.3f})")
