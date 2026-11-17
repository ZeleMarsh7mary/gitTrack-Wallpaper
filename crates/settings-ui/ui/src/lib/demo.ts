import type { UiConfig } from "./types";

/**
 * Browser-only mock renderer. Produces an SVG data URL that approximates what
 * the Rust renderer would draw (gradient/image/solid background, 53x7 heatmap
 * card). Used by the demo backend in ipc.ts so the preview is never empty
 * outside Tauri. Demo data, not real contribution data.
 */

const W = 960;
const H = 540;
const S = 0.5; // preview works at half resolution, viewBox is scaled up
const COLS = 53;
const ROWS = 7;

function safeHex(hex: string, fallback: string): string {
  return /^#[0-9a-f]{6}$/i.test(hex.trim()) ? hex.trim() : fallback;
}

function parseHex(hex: string, fallback: string): [number, number, number] {
  const clean = safeHex(hex, fallback).slice(1);
  return [
    parseInt(clean.slice(0, 2), 16),
    parseInt(clean.slice(2, 4), 16),
    parseInt(clean.slice(4, 6), 16),
  ];
}

function rgba(hex: string, alpha: number, fallback = "#101010"): string {
  const [r, g, b] = parseHex(hex, fallback);
  return `rgba(${r},${g},${b},${alpha.toFixed(3)})`;
}

function mix(hex: string, other: string, amount: number, fallback: string): string {
  const a = parseHex(hex, fallback);
  const b = parseHex(other, fallback);
  const channel = (index: number) =>
    Math.round(a[index] * (1 - amount) + b[index] * amount)
      .toString(16)
      .padStart(2, "0");
  return `#${channel(0)}${channel(1)}${channel(2)}`;
}

/** Deterministic pseudo-activity so the mock grid is stable across renders. */
function levelAt(col: number, row: number): number {
  const noise = Math.sin(col * 12.9898 + row * 78.233) * 43758.5453;
  const unit = noise - Math.floor(noise);
  const weekday = [1, 0.96, 0.9, 0.84, 0.72, 0.4, 0.3][row] ?? 0.6;
  const wave = 0.5 + 0.5 * Math.sin(col / 5.5 + 1.2);
  const score = unit * weekday * wave;
  if (score > 0.55) return 4;
  if (score > 0.42) return 3;
  if (score > 0.27) return 2;
  if (score > 0.13) return 1;
  return 0;
}

function backgroundLayer(cfg: UiConfig): string {
  const primary = safeHex(cfg.background.colorPrimary, "#0b1220");
  const secondary = safeHex(cfg.background.colorSecondary, "#12261c");
  const glow = mix(primary, "#ffffff", 0.22, "#0b1220");

  if (cfg.background.mode === "solid") {
    return `<rect width="${W}" height="${H}" fill="${primary}"/>`;
  }

  if (cfg.background.mode === "image") {
    return `
      <rect width="${W}" height="${H}" fill="${primary}"/>
      <g filter="url(#soften)">
        <ellipse cx="240" cy="140" rx="300" ry="200" fill="${rgba(secondary, 0.85, "#12261c")}"/>
        <ellipse cx="760" cy="430" rx="330" ry="240" fill="${rgba(glow, 0.35, "#20303f")}"/>
        <ellipse cx="520" cy="250" rx="260" ry="150" fill="${rgba("#ffffff", 0.05, "#ffffff")}"/>
      </g>
      <rect width="${W}" height="${H}" fill="url(#vignette)"/>`;
  }

  return `
    <rect width="${W}" height="${H}" fill="url(#bg-gradient)"/>
    <rect width="${W}" height="${H}" fill="url(#bg-glow)"/>`;
}

function cardAnchor(cfg: UiConfig): { x: number; y: number } {
  const base = {
    center: { x: 0.5, y: 0.5 },
    topLeft: { x: 0.3, y: 0.3 },
    topRight: { x: 0.7, y: 0.3 },
    bottomLeft: { x: 0.3, y: 0.7 },
    bottomRight: { x: 0.7, y: 0.7 },
    custom: { x: 0.5, y: 0.5 },
  }[cfg.card.position];
  return {
    x: base.x * W + cfg.card.offsetX * S,
    y: base.y * H + cfg.card.offsetY * S,
  };
}

function cardLayer(cfg: UiConfig): string {
  let box = cfg.track.boxSize * S;
  let gap = cfg.track.boxGap * S;
  let radius = Math.min(cfg.track.boxRadius * S, box / 2);

  const maxGridWidth = W * 0.62;
  const rawWidth = COLS * box + (COLS - 1) * gap;
  if (rawWidth > maxGridWidth) {
    const fit = maxGridWidth / rawWidth;
    box *= fit;
    gap *= fit;
    radius *= fit;
  }

  const gridW = COLS * box + (COLS - 1) * gap;
  const gridH = ROWS * box + (ROWS - 1) * gap;
  const pad = Math.max(10, box * 1.9);
  const headerH = cfg.track.showStatsHeader ? Math.max(13, box * 1.9) : 0;
  const cardW = gridW + pad * 2;
  const cardH = gridH + pad * 2 + headerH;

  const anchor = cardAnchor(cfg);
  const cardX = Math.min(Math.max(anchor.x - cardW / 2, 20), W - cardW - 20);
  const cardY = Math.min(Math.max(anchor.y - cardH / 2, 20), H - cardH - 20);
  const cardR = Math.min(cfg.card.cornerRadius * S, cardH / 2);
  const tint = safeHex(cfg.card.tintColor, "#10161f");
  const opacity = Math.min(Math.max(cfg.card.opacity, 0), 1);
  const text = safeHex(cfg.track.textColor, "#e6edf3");

  const palette = cfg.track.colors.map((color, index) =>
    safeHex(color, ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"][index] ?? "#161b22"),
  );

  let cardFill = "";
  if (cfg.card.material !== "none") {
    const strength =
      cfg.card.material === "solid" ? Math.max(opacity, 0.82) : Math.min(opacity, 0.92);
    cardFill = `
      <rect x="${cardX}" y="${cardY + 4}" width="${cardW}" height="${cardH}" rx="${cardR}" fill="rgba(0,0,0,0.28)"/>
      <rect x="${cardX}" y="${cardY}" width="${cardW}" height="${cardH}" rx="${cardR}" fill="${rgba(tint, strength, "#10161f")}" stroke="rgba(255,255,255,0.16)" stroke-width="1"/>`;
    if (cfg.card.material !== "solid") {
      cardFill += `<rect x="${cardX + 1}" y="${cardY + 1}" width="${cardW - 2}" height="${Math.min(cardH * 0.45, 28)}" rx="${Math.max(cardR - 1, 0)}" fill="rgba(255,255,255,0.05)"/>`;
    }
  } else {
    cardFill = `<rect x="${cardX}" y="${cardY}" width="${cardW}" height="${cardH}" rx="${cardR}" fill="rgba(0,0,0,0.22)"/>`;
  }

  let header = "";
  if (cfg.track.showStatsHeader) {
    const fontSize = Math.max(8, box * 1.15);
    header = `<text x="${cardX + pad}" y="${cardY + pad + fontSize * 0.85}" font-family="Segoe UI, sans-serif" font-size="${fontSize}" fill="${text}" fill-opacity="0.86">1,284 contributions &#183; 12 day streak</text>`;
  }

  const cells: string[] = [];
  for (let row = 0; row < ROWS; row += 1) {
    for (let col = 0; col < COLS; col += 1) {
      const x = cardX + pad + col * (box + gap);
      const y = cardY + pad + headerH + row * (box + gap);
      cells.push(
        `<rect x="${x.toFixed(2)}" y="${y.toFixed(2)}" width="${box.toFixed(2)}" height="${box.toFixed(2)}" rx="${radius.toFixed(2)}" fill="${palette[levelAt(col, row)]}"/>`,
      );
    }
  }

  return `${cardFill}${header}${cells.join("")}`;
}

export function demoPreview(cfg: UiConfig): string {
  const primary = safeHex(cfg.background.colorPrimary, "#0b1220");
  const secondary = safeHex(cfg.background.colorSecondary, "#12261c");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W * 2}" height="${H * 2}" viewBox="0 0 ${W} ${H}" role="img">
    <defs>
      <linearGradient id="bg-gradient" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="${primary}"/>
        <stop offset="1" stop-color="${secondary}"/>
      </linearGradient>
      <radialGradient id="bg-glow" cx="0.22" cy="0.12" r="0.85">
        <stop offset="0" stop-color="${mix(primary, "#ffffff", 0.24, "#0b1220")}" stop-opacity="0.5"/>
        <stop offset="1" stop-color="${primary}" stop-opacity="0"/>
      </radialGradient>
      <radialGradient id="vignette" cx="0.5" cy="0.5" r="0.75">
        <stop offset="0.55" stop-color="#000000" stop-opacity="0"/>
        <stop offset="1" stop-color="#000000" stop-opacity="0.45"/>
      </radialGradient>
      <filter id="soften" x="-30%" y="-30%" width="160%" height="160%">
        <feGaussianBlur stdDeviation="34"/>
      </filter>
    </defs>
    ${backgroundLayer(cfg)}
    ${cardLayer(cfg)}
  </svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}
