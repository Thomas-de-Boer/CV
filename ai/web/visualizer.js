// Draws the network: input grid, one circle per neuron, the strongest
// connections, and the output layer with probability bars. Canvas 2D only.

// Colours of the CV site. The network panel is dark in both themes.
import { t, formatNumber } from "./i18n.js";

const WARM = [138, 168, 255]; // active, pushes a neuron on
const COOL = [224, 112, 112]; // holds a neuron back
const ACCENT = [255, 209, 102]; // the answer
const OFF = [44, 44, 44];
const FONT = 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif';
const REVEAL_MS = 900;
const TOP_CONNECTIONS = 3;
const HOVER_CONNECTIONS = 6;

const clamp01 = (v) => Math.min(1, Math.max(0, v));
const smooth = (t) => t * t * (3 - 2 * t);
const rgba = (c, a) => `rgba(${c[0]},${c[1]},${c[2]},${a})`;
const mix = (a, b, t) => [0, 1, 2].map((i) => Math.round(a[i] + (b[i] - a[i]) * t));
const escapeHtml = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

export class NetworkView {
  /**
   * @param {HTMLCanvasElement} canvas
   * @param {HTMLElement} tooltip
   * @param {{sizes: number[], weights: Float32Array[], names?: string[][], titles?: string[]}} model
   */
  constructor(canvas, tooltip, model) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.tooltip = tooltip;
    this.model = model;
    this.input = null; // Float32Array(784) or null when nothing is drawn
    this.layers = null; // activations per layer after the input
    this.connections = []; // per transition
    this.reveal = model.sizes.length; // everything visible
    this.animStart = null;
    this.hover = null; // { layer (1-based), index }
    this.frame = 0;
    this.reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    new ResizeObserver(() => this.resize()).observe(canvas.parentElement);
    canvas.addEventListener("pointermove", (e) => this.onPointer(e));
    canvas.addEventListener("pointerdown", (e) => this.onPointer(e));
    canvas.addEventListener("pointerleave", (e) => {
      if (e.pointerType === "mouse") this.setHover(null);
    });
    this.resize();
  }

  /** Swap in another set of weights (possibly with other layer sizes). */
  setModel(model) {
    this.model = model;
    this.hover = null;
    this.clear();
    this.resize();
  }

  /** The language changed: redo the labels and the open tooltip. */
  relabel() {
    this.refreshTooltip();
    this.requestDraw();
  }

  // ------------------------------------------------------------ layout

  resize() {
    const width = Math.max(280, Math.floor(this.canvas.parentElement.clientWidth));
    const narrow = width < 560;
    const height = narrow ? 400 : 460;
    const dpr = window.devicePixelRatio || 1;
    this.w = width;
    this.h = height;
    this.dpr = dpr;
    this.narrow = narrow;
    this.canvas.style.height = `${height}px`;
    this.canvas.width = Math.round(width * dpr);
    this.canvas.height = Math.round(height * dpr);

    const top = 34;
    const usable = height - top - 12;
    const midY = top + usable / 2;
    const { sizes } = this.model;

    const cell = Math.max(2, Math.floor(Math.min(narrow ? 84 : 168, width * 0.26) / 28));
    this.grid = { x: narrow ? 6 : 12, y: midY - cell * 14, cell, size: cell * 28 };

    const outRoom = narrow ? 70 : 132;
    const outX = width - outRoom;
    const left = this.grid.x + this.grid.size;
    const hidden = sizes.length - 2;

    // positions[k] = neuron centres of layer k (1-based; 0 is the input grid)
    this.positions = [null];
    this.radius = [0];
    this.columnX = [this.grid.x + this.grid.size / 2];
    for (let k = 1; k < sizes.length; k++) {
      const n = sizes[k];
      const isOutput = k === sizes.length - 1;
      const x = isOutput ? outX : left + ((outX - left) * k) / (hidden + 1);
      const minSpacing = narrow ? 11 : 14;
      const perColumn = Math.max(1, Math.floor(usable / minSpacing));
      const columns = isOutput ? 1 : Math.ceil(n / perColumn);
      const rows = Math.ceil(n / columns);
      const spacing = Math.min(usable / rows, isOutput ? 40 : 24);
      const radius = isOutput ? Math.min(12, spacing * 0.38) : Math.min(7, spacing * 0.4);
      const columnGap = Math.max(radius * 3.4, spacing * 1.1);
      const points = [];
      for (let i = 0; i < n; i++) {
        const column = Math.floor(i / rows);
        const row = i % rows;
        const inColumn = Math.min(rows, n - column * rows);
        points.push({
          x: x + (column - (columns - 1) / 2) * columnGap,
          y: midY + (row - (inColumn - 1) / 2) * spacing,
        });
      }
      this.positions.push(points);
      this.radius.push(radius);
      this.columnX.push(x);
    }
    this.requestDraw();
  }

  pixelPosition(i) {
    const { x, y, cell } = this.grid;
    return { x: x + ((i % 28) + 0.5) * cell, y: y + (Math.floor(i / 28) + 0.5) * cell };
  }

  sourcePosition(transition, i) {
    return transition === 0 ? this.pixelPosition(i) : this.positions[transition][i];
  }

  // ------------------------------------------------------------- state

  /** Show a prediction. `animate` reveals the layers one after another. */
  show(input, layers, animate) {
    this.input = input;
    this.layers = layers;
    this.connections = layers.map((_, t) => this.strongestConnections(t));
    if (animate && !this.reducedMotion) {
      this.reveal = 0;
      this.animStart = performance.now();
    } else if (this.animStart === null) {
      this.reveal = this.model.sizes.length;
    }
    this.refreshTooltip();
    this.requestDraw();
  }

  /** Nothing drawn: an idle network. */
  clear() {
    this.input = null;
    this.layers = null;
    this.connections = [];
    this.animStart = null;
    this.reveal = this.model.sizes.length;
    this.refreshTooltip();
    this.requestDraw();
  }

  /** Activations feeding transition `t` (0 = from the input pixels). */
  activationsBefore(t) {
    return t === 0 ? this.input : this.layers[t - 1];
  }

  /** The `count` largest |weight × activation| going into one neuron. */
  contributions(t, target, count, positiveOnly = false) {
    const from = this.activationsBefore(t);
    const weights = this.model.weights[t];
    const n = this.model.sizes[t];
    const best = [];
    for (let i = 0; i < n; i++) {
      const value = weights[target * n + i] * (from ? from[i] : 1);
      if (positiveOnly ? value <= 0 : value === 0) continue;
      const size = Math.abs(value);
      if (best.length < count || size > best[best.length - 1].size) {
        best.push({ source: i, value, size });
        best.sort((a, b) => b.size - a.size);
        if (best.length > count) best.pop();
      }
    }
    return best;
  }

  strongestConnections(t) {
    const acts = this.layers[t];
    const isOutput = t === this.layers.length - 1;
    const max = Math.max(...acts);
    const list = [];
    for (let j = 0; j < acts.length; j++) {
      // Only neurons that are actually doing something get connections.
      if (isOutput ? acts[j] < 0.05 : acts[j] < 0.15 * max || acts[j] <= 0) continue;
      for (const c of this.contributions(t, j, TOP_CONNECTIONS, t === 0)) {
        list.push({ t, source: c.source, target: j, value: c.value, size: c.size });
      }
    }
    const strongest = Math.max(1e-6, ...list.map((c) => c.size));
    for (const c of list) c.strength = c.size / strongest;
    return list;
  }

  // -------------------------------------------------------------- hover

  onPointer(e) {
    const rect = this.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    let best = null;
    let bestDistance = e.pointerType === "mouse" ? 14 : 24;
    for (let k = 1; k < this.positions.length; k++) {
      this.positions[k].forEach((p, index) => {
        const d = Math.hypot(p.x - x, p.y - y);
        if (d < bestDistance) {
          bestDistance = d;
          best = { layer: k, index };
        }
      });
    }
    this.setHover(best);
  }

  setHover(hover) {
    const same = hover && this.hover && hover.layer === this.hover.layer && hover.index === this.hover.index;
    if (same || (!hover && !this.hover)) return;
    this.hover = hover;
    this.refreshTooltip();
    this.requestDraw();
  }

  neuronName(layer, index) {
    const isOutput = layer === this.model.sizes.length - 1;
    const name = this.model.names?.[layer - 1]?.[index];
    if (isOutput) return t("theDigit", { d: index });
    return name ?? t("neuron", { n: index + 1 });
  }

  refreshTooltip() {
    const tip = this.tooltip;
    if (!this.hover) {
      tip.hidden = true;
      return;
    }
    const { layer, index } = this.hover;
    const t = layer - 1;
    const isOutput = layer === this.model.sizes.length - 1;
    const active = this.layers !== null;
    const value = active ? this.layers[t][index] : null;
    const title = this.model.titles?.[t];

    let html = `<div class="name">${escapeHtml(this.neuronName(layer, index))}</div>`;
    html += `<div class="sub">${title ? escapeHtml(title) + " · " : ""}`;
    if (!active) html += t("tipIdle");
    else if (isOutput) html += t("tipSure", { pct: formatNumber(value * 100, 1) });
    else html += t("tipActivation", { value: formatNumber(value, 2) });
    html += "</div>";

    if (t === 0) {
      html += `<canvas width="28" height="28"></canvas>`;
      html += `<div class="sub"><span class="plus">■</span> ${t("tipWantsInk")} · <span class="minus">■</span> ${t("tipWantsEmpty")}</div>`;
    } else {
      const top = this.contributions(t, index, 5);
      if (top.length) {
        html += `<div class="sub" style="margin-top:6px">${t(active ? "tipStrongest" : "tipListens")}</div><ul>`;
        for (const c of top) {
          const sign = c.value >= 0 ? "plus" : "minus";
          const number = `${c.value >= 0 ? "+" : "−"}${formatNumber(Math.abs(c.value), 2)}`;
          html += `<li><span>${escapeHtml(this.neuronName(t, c.source))}</span><span class="${sign}">${number}</span></li>`;
        }
        html += "</ul>";
      }
    }
    tip.innerHTML = html;
    tip.hidden = false;
    if (t === 0) this.paintWeightMap(tip.querySelector("canvas"), index);

    // Place it next to the neuron, inside the canvas.
    const p = this.positions[layer][index];
    const tw = tip.offsetWidth;
    const th = tip.offsetHeight;
    let left = p.x + 16;
    if (left + tw > this.w - 4) left = p.x - 16 - tw;
    left = Math.max(4, Math.min(left, this.w - tw - 4));
    const topPx = Math.max(4, Math.min(p.y - th / 2, this.h - th - 4));
    tip.style.left = `${left}px`;
    tip.style.top = `${topPx}px`;
  }

  /** The 28×28 weight pattern of a first-layer neuron, with the drawing on top. */
  paintWeightMap(canvas, index) {
    const weights = this.model.weights[0];
    const offset = index * 784;
    let max = 1e-6;
    for (let i = 0; i < 784; i++) max = Math.max(max, Math.abs(weights[offset + i]));
    const ctx = canvas.getContext("2d");
    const image = ctx.createImageData(28, 28);
    for (let i = 0; i < 784; i++) {
      const w = weights[offset + i] / max;
      const strength = Math.pow(Math.abs(w), 0.6);
      let colour = mix([15, 15, 15], w >= 0 ? WARM : COOL, strength);
      if (this.input && this.input[i] > 0.3) colour = mix(colour, [255, 255, 255], 0.55 * this.input[i]);
      image.data.set([colour[0], colour[1], colour[2], 255], i * 4);
    }
    ctx.putImageData(image, 0, 0);
  }

  // ------------------------------------------------------------ drawing

  requestDraw() {
    if (this.frame) return;
    this.frame = requestAnimationFrame((now) => {
      this.frame = 0;
      if (this.animStart !== null) {
        const transitions = this.model.sizes.length - 1;
        const progress = (now - this.animStart) / REVEAL_MS;
        this.reveal = Math.min(1, Math.max(0, progress)) * transitions;
        if (progress >= 1) {
          this.animStart = null;
          this.reveal = this.model.sizes.length;
        } else {
          this.requestDraw();
        }
      }
      this.draw();
    });
  }

  /** How far layer `k` (1-based) has lit up during the reveal animation. */
  layerFade(k) {
    return smooth(clamp01((this.reveal - (k - 1) - 0.45) / 0.55));
  }

  draw() {
    const { ctx, w, h, dpr } = this;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    this.drawTitles();
    this.drawInputGrid();
    this.drawConnections();
    for (let k = 1; k < this.positions.length; k++) {
      if (k === this.positions.length - 1) this.drawOutput(k);
      else this.drawHidden(k);
    }
  }

  drawTitles() {
    const { ctx } = this;
    const { sizes, titles } = this.model;
    ctx.font = `${this.narrow ? 10 : 12}px ${FONT}`;
    ctx.fillStyle = "#a3a3a0";
    ctx.textBaseline = "top";
    ctx.textAlign = "center";
    for (let k = 0; k < sizes.length; k++) {
      let label;
      if (k === 0) label = t(this.narrow ? "netInputShort" : "netInput");
      else if (k === sizes.length - 1) label = t("netAnswer");
      else if (titles?.[k - 1]) label = this.narrow ? titles[k - 1].split(" ")[0] : titles[k - 1];
      else label = t(this.narrow ? "netHiddenShort" : "netHidden", { k });
      ctx.fillText(label, this.columnX[k], 6);
    }
  }

  drawInputGrid() {
    const { ctx, grid } = this;
    ctx.fillStyle = "#1a1a1a";
    ctx.fillRect(grid.x, grid.y, grid.size, grid.size);
    if (this.input) {
      for (let i = 0; i < 784; i++) {
        const v = this.input[i];
        if (v <= 0.02) continue;
        ctx.fillStyle = rgba([236, 236, 234], v);
        ctx.fillRect(grid.x + (i % 28) * grid.cell, grid.y + Math.floor(i / 28) * grid.cell, grid.cell, grid.cell);
      }
    }
    ctx.strokeStyle = "#2c2c2c";
    ctx.lineWidth = 1;
    ctx.strokeRect(grid.x - 0.5, grid.y - 0.5, grid.size + 1, grid.size + 1);
  }

  drawConnections() {
    const { ctx } = this;
    const hover = this.hover;
    ctx.lineCap = "round";
    for (const list of this.connections) {
      for (const c of list) {
        const travelled = smooth(clamp01(this.reveal - c.t));
        if (travelled <= 0) continue;
        const hovered = hover && hover.layer === c.t + 1 && hover.index === c.target;
        if (hovered) continue; // drawn below, in full
        const alpha = (c.t === 0 ? 0.28 : 0.6) * c.strength * (hover ? 0.25 : 1);
        this.line(c, travelled, alpha, 1 + 1.5 * c.strength);
      }
    }
    if (hover) {
      const t = hover.layer - 1;
      const list = this.contributions(t, hover.index, HOVER_CONNECTIONS);
      const strongest = Math.max(1e-6, ...list.map((c) => c.size));
      for (const c of list) {
        this.line({ t, source: c.source, target: hover.index, value: c.value }, 1, 0.35 + 0.6 * (c.size / strongest), 2);
      }
    }
  }

  line(c, travelled, alpha, width) {
    const { ctx } = this;
    const from = this.sourcePosition(c.t, c.source);
    const to = this.positions[c.t + 1][c.target];
    ctx.strokeStyle = rgba(c.value >= 0 ? WARM : COOL, alpha);
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(from.x + (to.x - from.x) * travelled, from.y + (to.y - from.y) * travelled);
    ctx.stroke();
  }

  drawHidden(k) {
    const { ctx } = this;
    const acts = this.layers?.[k - 1];
    // Brightness is relative to the busiest neuron of the layer.
    const max = acts ? Math.max(0.5, ...acts) : 1;
    const fade = this.layerFade(k);
    const r = this.radius[k];
    this.positions[k].forEach((p, i) => {
      const level = acts ? clamp01(acts[i] / max) * fade : 0;
      const hovered = this.hover && this.hover.layer === k && this.hover.index === i;
      if (level > 0.25) {
        ctx.fillStyle = rgba(WARM, 0.16 * level);
        ctx.beginPath();
        ctx.arc(p.x, p.y, r * 1.9, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.fillStyle = rgba(mix(OFF, level > 0.85 ? mix(WARM, [255, 255, 255], (level - 0.85) * 3) : WARM, Math.min(1, level / 0.85)), 1);
      ctx.strokeStyle = hovered ? "#ffffff" : level > 0.05 ? rgba(WARM, 0.9) : "#4a4a4a";
      ctx.lineWidth = hovered ? 2 : 1;
      ctx.beginPath();
      ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    });
  }

  drawOutput(k) {
    const { ctx } = this;
    const acts = this.layers?.[k - 1];
    const fade = this.layerFade(k);
    const r = this.radius[k];
    const best = acts ? acts.indexOf(Math.max(...acts)) : -1;
    const barWidth = this.narrow ? 30 : 64;
    this.positions[k].forEach((p, i) => {
      const prob = acts ? acts[i] * fade : 0;
      const winner = i === best && fade > 0.6;
      const hovered = this.hover && this.hover.layer === k && this.hover.index === i;

      ctx.fillStyle = rgba(mix(OFF, winner ? ACCENT : WARM, clamp01(prob)), 1);
      ctx.strokeStyle = hovered ? "#ffffff" : winner ? rgba(ACCENT, 1) : "#4a4a4a";
      ctx.lineWidth = winner || hovered ? 2.5 : 1;
      ctx.beginPath();
      ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      ctx.font = `${winner ? "700" : "500"} ${Math.round(r * 1.15)}px ${FONT}`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillStyle = prob > 0.55 ? "#10131c" : winner ? "#ffffff" : "#a3a3a0";
      ctx.fillText(String(i), p.x, p.y + 0.5);

      const bx = p.x + r + 8;
      ctx.fillStyle = "#1a1a1a";
      ctx.fillRect(bx, p.y - 3, barWidth, 6);
      ctx.fillStyle = winner ? rgba(ACCENT, 1) : rgba(WARM, 0.55);
      ctx.fillRect(bx, p.y - 3, barWidth * clamp01(prob), 6);
      if (!this.narrow && acts) {
        ctx.font = `11px ${FONT}`;
        ctx.textAlign = "left";
        ctx.fillStyle = winner ? "#ececea" : "#a3a3a0";
        ctx.fillText(`${Math.round(prob * 100)}%`, bx + barWidth + 6, p.y + 0.5);
      }
    });
  }
}
