import type { UiConfig } from "./types";

export interface PalettePreset {
  name: string;
  colors: [string, string, string, string, string];
}

/** Built-in ramps, mirroring src/config.rs PALETTES. */
export const PALETTES: PalettePreset[] = [
  {
    name: "GitHub Dark",
    colors: ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"],
  },
  {
    name: "GitHub Light",
    colors: ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"],
  },
  {
    name: "Dracula",
    colors: ["#21222c", "#3a3f58", "#6272a4", "#bd93f9", "#ff79c6"],
  },
  {
    name: "Monochrome",
    colors: ["#1b1b1b", "#3d3d3d", "#6b6b6b", "#9d9d9d", "#f0f0f0"],
  },
  {
    name: "Neon",
    colors: ["#101820", "#0b3d2e", "#128a5a", "#22d38a", "#8affc1"],
  },
];

export const DEFAULT_CONFIG: UiConfig = {
  source: {
    mode: "profile",
    inputTarget: "octocat",
    apiKey: "",
    refreshIntervalMinutes: 30,
  },
  background: {
    mode: "gradient",
    imagePath: "",
    fit: "cover",
    colorPrimary: "#0b1220",
    colorSecondary: "#12261c",
  },
  card: {
    position: "center",
    offsetX: 0,
    offsetY: 0,
    material: "glass",
    tintColor: "#10161f",
    opacity: 0.55,
    blurRadius: 24,
    cornerRadius: 20,
  },
  track: {
    boxSize: 14,
    boxGap: 4,
    boxRadius: 3,
    colors: ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"],
    showStatsHeader: true,
    textColor: "#e6edf3",
  },
  startWithWindows: false,
};

export function paletteFor(colors: readonly string[]): PalettePreset | null {
  return (
    PALETTES.find((preset) =>
      preset.colors.every(
        (color, index) => color.toLowerCase() === (colors[index] ?? "").toLowerCase(),
      ),
    ) ?? null
  );
}

/** Normalise a hex string for <input type="color">, which only accepts #rrggbb. */
export function toColorInputValue(hex: string, fallback: string): string {
  return /^#[0-9a-f]{6}$/i.test(hex.trim()) ? hex.trim().toLowerCase() : fallback;
}
