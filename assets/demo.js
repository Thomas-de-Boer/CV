// The small neural-network demo on the CV page. It uses the same Rust →
// WebAssembly network and trained weights as the full demo in ../ai/web.

import init, { Model, preprocess } from "../ai/web/pkg/nn_wasm.js";

const WEIGHTS = new URL("../ai/web/weights-trained.bin", import.meta.url);
const PAD = 280; // canvas pixels (shown at 200 CSS px)
const BRUSH = 24;
const INK = "#ececea";
const LIVE_INTERVAL_MS = 100;
// The "3" from the design, shown until the visitor starts drawing.
const SAMPLE = "M28 30c10-16 54-20 62 2 6 18-14 30-34 36 24 2 40 14 36 34-6 26-52 28-66 10";
const DOT_OFF = [44, 44, 44];
const DOT_ON = [138, 168, 255];

const pad = document.getElementById("demo-canvas");
const ctx = pad.getContext("2d", { willReadFrequently: true });
const layersEl = document.getElementById("demo-layers");
const barsEl = document.getElementById("demo-bars");
const guessEl = document.getElementById("demo-guess");

let model = null;
let sizes = [];
let seenCtx = null;
let dotColumns = []; // per layer after the input: its dot elements
let bars = []; // per digit: { row, fill }
let hasInk = false;
let showingSample = false;

// ---------------------------------------------------------------- drawing

function resetPad() {
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, PAD, PAD);
  ctx.strokeStyle = INK;
  ctx.fillStyle = INK;
  ctx.lineWidth = BRUSH;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  hasInk = false;
  showingSample = false;
}

function drawSample() {
  resetPad();
  ctx.save();
  ctx.translate((PAD - 120 * 1.4) / 2, (PAD - 140 * 1.4) / 2);
  ctx.scale(1.4, 1.4);
  ctx.lineWidth = BRUSH / 1.4;
  ctx.stroke(new Path2D(SAMPLE));
  ctx.restore();
  hasInk = true;
  showingSample = true;
}

function padPoint(e) {
  const rect = pad.getBoundingClientRect();
  return {
    x: ((e.clientX - rect.left) / rect.width) * PAD,
    y: ((e.clientY - rect.top) / rect.height) * PAD,
  };
}

let stroke = null; // last point of the stroke being drawn
let liveTimer = 0;

pad.addEventListener("pointerdown", (e) => {
  if (e.button !== 0 && e.pointerType === "mouse") return;
  e.preventDefault();
  try {
    pad.setPointerCapture(e.pointerId); // keep the stroke when the pointer leaves the pad
  } catch {
    // Not fatal: the stroke just ends at the edge.
  }
  if (showingSample) resetPad();
  stroke = padPoint(e);
  // A dot, so a single tap leaves ink too.
  ctx.beginPath();
  ctx.arc(stroke.x, stroke.y, BRUSH / 2, 0, Math.PI * 2);
  ctx.fill();
  hasInk = true;
  scheduleLive();
});

pad.addEventListener("pointermove", (e) => {
  if (!stroke) return;
  e.preventDefault();
  const events = e.getCoalescedEvents?.() ?? [];
  for (const ev of events.length ? events : [e]) {
    const p = padPoint(ev);
    ctx.beginPath();
    ctx.moveTo(stroke.x, stroke.y);
    ctx.lineTo(p.x, p.y);
    ctx.stroke();
    stroke = p;
  }
  scheduleLive();
});

function endStroke() {
  if (!stroke) return;
  stroke = null;
  clearTimeout(liveTimer);
  liveTimer = 0;
  predict();
}
pad.addEventListener("pointerup", endStroke);
pad.addEventListener("pointercancel", endStroke);

/** While drawing: predict at most every 100 ms. */
function scheduleLive() {
  if (liveTimer) return;
  liveTimer = setTimeout(() => {
    liveTimer = 0;
    predict();
  }, LIVE_INTERVAL_MS);
}

document.getElementById("demo-clear").addEventListener("click", () => {
  resetPad();
  predict();
});

// ---------------------------------------------------------------- display

function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/** One column per layer: the 28×28 input image, then a dot per neuron. */
function buildLayers() {
  layersEl.replaceChildren();
  dotColumns = [];

  const input = element("div", "demo-layer");
  const wrap = element("div", "demo-seen-wrap");
  const seen = element("canvas");
  seen.id = "demo-seen";
  seen.width = 28;
  seen.height = 28;
  seenCtx = seen.getContext("2d");
  wrap.append(seen);
  input.append(wrap, element("span", "demo-caption", String(sizes[0])));
  layersEl.append(input);

  for (const n of sizes.slice(1)) {
    const column = element("div", "demo-layer");
    const dots = element("div", n <= 10 ? "demo-dots is-large" : "demo-dots");
    dots.style.gridTemplateRows = `repeat(${Math.min(n, 16)}, auto)`;
    const list = Array.from({ length: n }, () => element("span"));
    dots.append(...list);
    dotColumns.push(list);
    column.append(dots, element("span", "demo-caption", String(n)));
    layersEl.append(column);
  }

  barsEl.replaceChildren();
  bars = Array.from({ length: sizes[sizes.length - 1] }, (_, digit) => {
    const row = element("div", "demo-bar");
    const trackEl = element("div", "demo-bar-track");
    const fill = element("div", "demo-bar-fill");
    trackEl.append(fill);
    row.append(element("span", "", String(digit)), trackEl);
    barsEl.append(row);
    return { row, fill };
  });
}

function dotColor(level) {
  const c = DOT_OFF.map((off, i) => Math.round(off + (DOT_ON[i] - off) * level));
  return `rgb(${c[0]},${c[1]},${c[2]})`;
}

function show(pixels, layers) {
  const image = seenCtx.createImageData(28, 28);
  for (let i = 0; i < 784; i++) {
    const v = pixels ? pixels[i] : 0;
    // Same colours as the pad: ink on the dark surface.
    image.data.set([27 + v * 210, 26 + v * 210, 40 + v * 204, 255], i * 4);
  }
  seenCtx.putImageData(image, 0, 0);

  dotColumns.forEach((dots, l) => {
    const values = layers ? layers[l] : null;
    // Hidden layers have no fixed range, so each is scaled to its own strongest neuron.
    const max = values ? Math.max(...values, 1e-6) : 1;
    dots.forEach((dot, i) => {
      dot.style.background = dotColor(values ? Math.max(0, values[i]) / max : 0);
    });
  });

  const output = layers ? layers[layers.length - 1] : null;
  const top = output ? output.indexOf(Math.max(...output)) : -1;
  bars.forEach(({ row, fill }, digit) => {
    fill.style.width = output ? `${Math.round(output[digit] * 100)}%` : "0";
    row.classList.toggle("is-top", digit === top);
  });
  guessEl.textContent = output ? String(top) : "?";
  guessEl.classList.toggle("is-empty", !output);
}

// ------------------------------------------------------------- prediction

function predict() {
  if (!model) return;
  if (!hasInk) {
    show(null, null);
    return;
  }
  // The pad is transparent with light ink, so the alpha channel is the ink.
  const rgba = ctx.getImageData(0, 0, PAD, PAD).data;
  const ink = new Float32Array(PAD * PAD);
  for (let i = 0; i < ink.length; i++) ink[i] = rgba[i * 4 + 3] / 255;

  const pixels = preprocess(ink, PAD, PAD);
  const all = model.predict(pixels);
  const layers = [];
  let offset = 0;
  for (const n of sizes.slice(1)) {
    layers.push(all.subarray(offset, offset + n));
    offset += n;
  }
  show(pixels, layers);
}

// ------------------------------------------------------------------ start

async function start() {
  drawSample();
  try {
    const [, response] = await Promise.all([init(), fetch(WEIGHTS)]);
    if (!response.ok) throw new Error(`${WEIGHTS}: HTTP ${response.status}`);
    model = Model.from_bytes(new Uint8Array(await response.arrayBuffer()));
    sizes = [...model.layer_sizes()];
    buildLayers();
    predict();
  } catch (error) {
    console.error(error);
    document.getElementById("demo-error").hidden = false;
  }
}

start();
