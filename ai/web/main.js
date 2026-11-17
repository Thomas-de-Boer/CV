// Drawing, preprocessing and prediction. The network itself is Rust,
// compiled to WebAssembly (see ../nn-wasm and ../nn-core).

import init, { Model, preprocess } from "./pkg/nn_wasm.js";
import { NetworkView } from "./visualizer.js";
import { t, onLangChange } from "./i18n.js";

// The two sets of weights for the same network. "claude" always exists;
// "trained" is optional (the switch is hidden without it).
const MODELS = {
  claude: { file: "weights-guess.bin" },
  trained: { file: "weights-trained.bin" },
};
const DEFAULT_MODEL = "trained";
const PAD = 280;
const BRUSH = 24; // canvas pixels; a digit drawn ~200 px tall becomes ~2.5 px thick at 28×28
const LIVE_INTERVAL_MS = 100;
const INK = "#ececea";

const pad = document.getElementById("pad");
const padCtx = pad.getContext("2d", { willReadFrequently: true });
const seen = document.getElementById("seen");
const seenCtx = seen.getContext("2d");
const answerDigit = document.getElementById("answer-digit");
const answerDetail = document.getElementById("answer-detail");
const status = document.getElementById("status");

let model = null;
let modelId = null;
let view = null;
let sizes = [];
let hasInk = false;
let latest = null; // { pixels: Float32Array(784), layers: Float32Array[] }
let failure = null; // the error that stopped the network from loading, if any

// ---------------------------------------------------------------- drawing

function resetPad() {
  padCtx.clearRect(0, 0, PAD, PAD);
  padCtx.strokeStyle = INK;
  padCtx.fillStyle = INK;
  padCtx.lineWidth = BRUSH;
  padCtx.lineCap = "round";
  padCtx.lineJoin = "round";
  hasInk = false;
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
  stroke = padPoint(e);
  // A dot, so a single tap leaves ink too.
  padCtx.beginPath();
  padCtx.arc(stroke.x, stroke.y, BRUSH / 2, 0, Math.PI * 2);
  padCtx.fill();
  hasInk = true;
  scheduleLive();
});

pad.addEventListener("pointermove", (e) => {
  if (!stroke) return;
  e.preventDefault();
  // Coalesced events keep fast strokes smooth.
  const events = e.getCoalescedEvents?.() ?? [];
  for (const ev of events.length ? events : [e]) {
    const p = padPoint(ev);
    padCtx.beginPath();
    padCtx.moveTo(stroke.x, stroke.y);
    padCtx.lineTo(p.x, p.y);
    padCtx.stroke();
    stroke = p;
  }
  scheduleLive();
});

function endStroke() {
  if (!stroke) return;
  stroke = null;
  clearTimeout(liveTimer);
  liveTimer = 0;
  predict(true);
}
pad.addEventListener("pointerup", endStroke);
pad.addEventListener("pointercancel", endStroke);

/** While drawing: predict at most every 100 ms, without the animation. */
function scheduleLive() {
  if (liveTimer) return;
  liveTimer = setTimeout(() => {
    liveTimer = 0;
    predict(false);
  }, LIVE_INTERVAL_MS);
}

function clearAll() {
  resetPad();
  predict(false);
}
document.getElementById("clear").addEventListener("click", clearAll);

// ------------------------------------------------------------- prediction

function predict(animate) {
  if (!model) return;
  if (!hasInk) {
    latest = null;
    seenCtx.clearRect(0, 0, 28, 28);
    view.clear();
    showAnswer(null);
    return;
  }
  // The pad is transparent with light ink, so the alpha channel is the ink.
  const rgba = padCtx.getImageData(0, 0, PAD, PAD).data;
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
  latest = { pixels, layers };

  const image = seenCtx.createImageData(28, 28);
  for (let i = 0; i < 784; i++) {
    image.data.set([236, 236, 234, Math.round(pixels[i] * 255)], i * 4);
  }
  seenCtx.putImageData(image, 0, 0);

  view.show(pixels, layers, animate);
  showAnswer(layers[layers.length - 1]);
}

function showAnswer(probabilities) {
  if (!probabilities) {
    answerDigit.textContent = "?";
    answerDigit.classList.add("unsure");
    answerDetail.textContent = t("drawToStart");
    return;
  }
  const ranked = [...probabilities.keys()].sort((a, b) => probabilities[b] - probabilities[a]);
  const [first, second] = ranked;
  const pct = (d) => `${Math.round(probabilities[d] * 100)}%`;
  answerDigit.textContent = String(first);
  answerDigit.classList.toggle("unsure", probabilities[first] < 0.5);
  let text = t(probabilities[first] < 0.5 ? "notSure" : "sure", { pct: pct(first) });
  if (probabilities[second] >= 0.05) text += t("couldBe", { digit: second, pct: pct(second) });
  answerDetail.textContent = text;
}

// ---------------------------------------------------------------- language

/** Everything this file writes into the page, in the current language. */
function showTexts() {
  if (failure) {
    status.textContent = t("loadFailed");
    answerDetail.textContent =
      location.protocol === "file:" ? t("loadFailedFile") : t("loadFailedWhy", { message: failure.message });
    return;
  }
  if (!model) return; // still loading: the page's own "Loading…" text stands
  status.textContent = t("status", { sizes: sizes.join(" → ") });
  showAnswer(latest ? latest.layers[latest.layers.length - 1] : null);
}

onLangChange(() => {
  showTexts();
  view?.relabel();
});

// ------------------------------------------------------------------- theme
// Shared with the CV page (same localStorage key).

const themeToggle = document.getElementById("theme-toggle");
const isDark = () => document.documentElement.classList.contains("t-dark");
themeToggle.setAttribute("aria-checked", String(isDark()));
themeToggle.addEventListener("click", () => {
  const next = isDark() ? "light" : "dark";
  document.documentElement.className = `t-${next}`;
  try {
    localStorage.setItem("cv-theme", next);
  } catch {
    // Private mode: the choice just isn't remembered.
  }
  themeToggle.setAttribute("aria-checked", String(isDark()));
  document.querySelector('meta[name="theme-color"]').content = isDark() ? "#151515" : "#faf9f6";
});

// ------------------------------------------------------------------- start

async function fetchOk(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  return response;
}

async function start() {
  resetPad();
  try {
    const [, claude, trained] = await Promise.all([
      init(),
      fetchOk(MODELS.claude.file),
      fetchOk(MODELS.trained.file).catch(() => null),
    ]);
    await loadModel("claude", claude);
    if (trained) await loadModel("trained", trained);

    const wanted = new URLSearchParams(location.search).get("model");
    const first = MODELS[wanted]?.model ? wanted : MODELS[DEFAULT_MODEL].model ? DEFAULT_MODEL : "claude";
    const loaded = MODELS[first];
    view = new NetworkView(document.getElementById("net"), document.getElementById("tooltip"), {
      sizes: loaded.sizes,
      weights: loaded.weights,
    });
    setupSwitch();
    selectModel(first, false);
    // The canvas labels use the site's font; redraw once it has loaded.
    document.fonts?.ready.then(() => view.requestDraw());
  } catch (error) {
    console.error(error);
    failure = error;
    showTexts();
    return;
  }
  if (new URLSearchParams(location.search).has("dev")) startDevMode();
}

async function loadModel(id, response) {
  const entry = MODELS[id];
  entry.model = Model.from_bytes(new Uint8Array(await response.arrayBuffer()));
  entry.sizes = [...entry.model.layer_sizes()];
  entry.weights = entry.sizes.slice(1).map((_, t) => entry.model.weights(t));
}

function setupSwitch() {
  const panel = document.getElementById("model-switch");
  if (!MODELS.trained.model) return; // only one set of weights: nothing to switch
  panel.hidden = false;
  for (const radio of panel.querySelectorAll('input[name="model"]')) {
    radio.addEventListener("change", () => {
      if (radio.checked) selectModel(radio.value, true);
    });
  }
}

/** Swap the weights; the drawing stays and is read again by the new ones. */
function selectModel(id, animate) {
  const entry = MODELS[id];
  if (!entry?.model) return;
  modelId = id;
  model = entry.model;
  sizes = entry.sizes;
  view.setModel({ sizes: entry.sizes, weights: entry.weights });

  for (const radio of document.querySelectorAll('input[name="model"]')) radio.checked = radio.value === id;
  for (const text of document.querySelectorAll("[data-model-text]")) text.hidden = text.dataset.modelText !== id;
  showTexts();

  const url = new URL(location.href);
  url.searchParams.set("model", id);
  history.replaceState(null, "", url);

  predict(animate);
}

// ---------------------------------------------------------------- dev mode
// ?dev=1 — collect labelled samples (a small real-world test set) and export
// them for `cargo run -p evaluate`.

function startDevMode() {
  const KEY = "digit-samples-v1";
  const panel = document.getElementById("dev");
  const count = document.getElementById("dev-count");
  const matrix = document.getElementById("dev-matrix");
  const digits = document.getElementById("dev-digits");
  panel.hidden = false;

  let samples = [];
  try {
    samples = JSON.parse(localStorage.getItem(KEY) ?? "[]");
  } catch {
    samples = [];
  }

  const store = () => {
    try {
      localStorage.setItem(KEY, JSON.stringify(samples));
    } catch (error) {
      console.warn("Could not store samples; export them now.", error);
    }
  };

  const refresh = () => {
    const right = samples.filter((s) => s.said === s.label).length;
    count.textContent = samples.length
      ? `${samples.length} samples · ${right} read correctly (${Math.round((100 * right) / samples.length)}%)`
      : "0 samples";
    const perDigit = Array.from({ length: 10 }, (_, d) => {
      const mine = samples.filter((s) => s.label === d);
      const ok = mine.filter((s) => s.said === d).length;
      return `${d}: ${ok}/${mine.length}`;
    });
    matrix.textContent = samples.length ? `correct per digit   ${perDigit.join("   ")}` : "";
  };

  const save = (label) => {
    if (!latest) return;
    const output = latest.layers[latest.layers.length - 1];
    samples.push({
      label,
      model: modelId,
      said: output.indexOf(Math.max(...output)),
      pixels: Array.from(latest.pixels, (v) => Math.round(v * 1000) / 1000),
    });
    store();
    refresh();
    clearAll();
  };

  for (let d = 0; d < 10; d++) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = String(d);
    button.title = `Save this drawing as a ${d}`;
    button.addEventListener("click", () => save(d));
    digits.append(button);
  }

  window.addEventListener("keydown", (e) => {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key >= "0" && e.key <= "9") save(Number(e.key));
    if (e.key === "Escape" || e.key === "Backspace") clearAll();
  });

  document.getElementById("dev-undo").addEventListener("click", () => {
    samples.pop();
    store();
    refresh();
  });

  document.getElementById("dev-reset").addEventListener("click", () => {
    if (!samples.length || !confirm(`Delete all ${samples.length} samples?`)) return;
    samples = [];
    store();
    refresh();
  });

  document.getElementById("dev-export").addEventListener("click", () => {
    const data = { samples: samples.map(({ label, pixels }) => ({ label, pixels })) };
    const url = URL.createObjectURL(new Blob([JSON.stringify(data)], { type: "application/json" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "samples.json";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  });

  refresh();
}

start();
