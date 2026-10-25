// Draws one frame of the demo film as a pure function of time (specs/005-demo-film).
// The renderer calls film.load(timeline) once, then film.seek(t) for every frame.
"use strict";

const U = 98, KX = (1920 - 14.5 * U) / 2, KY = 584, GAP = 7;
const ZHUYIN = {
  1: "ㄅ", 2: "ㄉ", 3: "ˇ", 4: "ˋ", 5: "ㄓ", 6: "ˊ", 7: "˙", 8: "ㄚ", 9: "ㄞ", 0: "ㄢ", "-": "ㄦ",
  q: "ㄆ", w: "ㄊ", e: "ㄍ", r: "ㄐ", t: "ㄔ", y: "ㄗ", u: "ㄧ", i: "ㄛ", o: "ㄟ", p: "ㄣ",
  a: "ㄇ", s: "ㄋ", d: "ㄎ", f: "ㄑ", g: "ㄕ", h: "ㄘ", j: "ㄨ", k: "ㄜ", l: "ㄠ", ";": "ㄤ",
  z: "ㄈ", x: "ㄌ", c: "ㄏ", v: "ㄒ", b: "ㄖ", n: "ㄙ", m: "ㄩ", ",": "ㄝ", ".": "ㄡ", "/": "ㄥ",
};
// Rows of [id, label, width in key units]; one-character ids are physical keys.
const ROWS = [
  [..."`1234567890-="].map((c) => [c, c, 1]).concat([["Backspace", "delete", 1.5]]),
  [["Tab", "tab", 1.5]].concat([..."qwertyuiop[]\\"].map((c) => [c, c, 1])),
  [["Caps", "caps lock", 1.75]].concat([..."asdfghjkl;'"].map((c) => [c, c, 1]), [["Enter", "return", 1.75]]),
  [["Shift", "shift", 2.25]].concat([..."zxcvbnm,./"].map((c) => [c, c, 1]), [["ShiftR", "shift", 2.25]]),
  [["Ctrl", "control", 1.5], ["Opt", "option", 1.25], ["Cmd", "command", 1.5], ["Space", "", 6],
   ["CmdR", "command", 1.5], ["OptR", "option", 1.25], ["CtrlR", "control", 1.5]],
];

const $ = (id) => document.getElementById(id);
const NS = "http://www.w3.org/2000/svg";
const keyEls = {};
let T = null;

function el(tag, attrs, parent) {
  const n = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs)) n.setAttribute(k, v);
  parent.appendChild(n);
  return n;
}

function drawKeyboard() {
  const svg = $("kbd");
  el("rect", { class: "case", x: KX - 22, y: KY - 22, width: 14.5 * U + 44, height: 5 * U + 44, rx: 26 }, svg);
  ROWS.forEach((row, r) => {
    let x = KX;
    for (const [id, label, w] of row) {
      const g = el("g", { class: "k" }, svg);
      const kx = x + GAP / 2, ky = KY + r * U + GAP / 2, kw = w * U - GAP, kh = U - GAP;
      el("rect", { class: "cap", x: kx, y: ky, width: kw, height: kh, rx: 9 }, g);
      el("rect", { class: "top", x: kx + 6, y: ky + 5, width: kw - 12, height: kh - 14, rx: 6 }, g);
      if (id.length === 1) {
        const lat = el("text", { class: "lat", x: kx + 13, y: ky + 30 }, g);
        lat.textContent = /[a-z]/.test(label) ? label.toUpperCase() : label;
        if (ZHUYIN[id]) {
          const sym = el("text", { class: "sym", x: kx + 13, y: ky + kh - 16 }, g);
          sym.textContent = ZHUYIN[id];
        }
      } else if (label) {
        const mod = el("text", { class: "mod", x: kx + 12, y: ky + kh - 16 }, g);
        mod.textContent = label;
      }
      keyEls[id] = g;
      x += w * U;
    }
  });
}

const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
const ease = (u) => { u = clamp(u, 0, 1); return u * u * (3 - 2 * u); };

function lastIndexAt(list, t) {
  let lo = -1, hi = list.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (list[mid].t <= t) lo = mid; else hi = mid - 1;
  }
  return lo;
}

function esc(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// Committed lines plus the live line (committed prefix, preedit, caret) at time t.
function renderDoc(t) {
  const i = lastIndexAt(T.events, t);
  const lines = [""];
  for (let k = 0; k <= i; k++) {
    const e = T.events[k];
    lines[lines.length - 1] += e.commit;
    if (e.code === "Enter" && !e.handled) lines.push("");
  }
  const cur = i >= 0 ? T.events[i] : { preedit: [], cursor: 0 };
  const idle = i < 0 || t - T.events[i].t > 0.6;
  const caretOn = !idle || Math.floor(t * 1.8) % 2 === 0;
  const caret = `<span class="caret" style="opacity:${caretOn ? 1 : 0}"></span>`;
  let live = esc(lines[lines.length - 1]);
  let pos = 0, placed = false;
  cur.preedit.forEach((seg, n) => {
    if (!placed && pos >= cur.cursor) { live += caret; placed = true; }
    const flash = i >= 0 && T.events[i].flash && T.events[i].flash.includes(n)
      ? `<span class="flash" style="opacity:${(0.55 * (1 - clamp((t - T.events[i].t) / 0.55, 0, 1))).toFixed(3)}"></span>` : "";
    live += `<span class="seg ${seg.kind}">${flash}${esc(seg.text)}</span>`;
    pos += [...seg.text].reduce((a, ch) => a + (ch.codePointAt(0) > 0xffff ? 2 : 1), 0);
  });
  if (!placed) live += caret;
  const html = lines.slice(0, -1).map((l) => `<div class="line">${esc(l)}</div>`);
  html.push(`<div class="line">${live}</div>`);
  $("doc").innerHTML = html.join("");
  return lines.length - 1;
}

function renderKeys(t) {
  for (const g of Object.values(keyEls)) g.classList.remove("down");
  for (const p of T.presses) {
    if (p.down <= t && t < p.up && keyEls[p.id]) keyEls[p.id].classList.add("down");
  }
}

function renderCaption(t) {
  let text = "", alpha = 0;
  for (const c of T.captions) {
    if (t >= c.t - 0.3 && t < c.end + 0.3) {
      text = c.text;
      alpha = Math.min(ease((t - c.t + 0.3) / 0.3), ease((c.end + 0.3 - t) / 0.3));
    }
  }
  $("caption").textContent = text;
  $("caption").style.opacity = alpha.toFixed(3);
}

function renderCamera(t, lineIndex) {
  let s = 1;
  for (const p of T.pushes) {
    const up = ease((t - p.t) / 0.8), down = 1 - ease((t - p.end) / 0.9);
    s = Math.max(s, 1 + (p.scale - 1) * Math.min(up, down));
  }
  const fy = 64 + 86 * lineIndex + 43;
  const stage = $("stage");
  stage.style.transformOrigin = `760px ${fy}px`;
  stage.style.transform = `scale(${s.toFixed(4)})`;
}

window.film = {
  load(timeline) {
    T = timeline;
    drawKeyboard();
    this.seek(0);
  },
  seek(t) {
    const line = renderDoc(t);
    renderKeys(t);
    renderCaption(t);
    renderCamera(t, line);
    $("end").style.opacity = ease((t - T.endCard) / 0.7).toFixed(3);
  },
  // Safe-margin and overlap check for QA (DF-8): returns problems at this time.
  qa() {
    const out = [];
    const doc = $("doc").getBoundingClientRect(), cap = $("caption").getBoundingClientRect();
    const scale = parseFloat(($("stage").style.transform.match(/scale\(([\d.]+)\)/) || [0, 1])[1]);
    if (scale === 1) {
      for (const line of document.querySelectorAll(".line")) {
        if (line.getBoundingClientRect().bottom > KY - 30) out.push("text overlaps keyboard");
        const range = document.createRange(); range.selectNodeContents(line);
        const box = range.getBoundingClientRect();
        if (box.right > 1920 - 64 || box.left < 64) out.push(`line outside margin: ${line.textContent}`);
      }
      if (doc.top < 40) out.push("document above margin");
      if ($("caption").textContent && cap.bottom > KY - 22) out.push("caption overlaps keyboard");
    }
    return out;
  },
};
