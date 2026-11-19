export type SourceMode = "profile" | "repository";
export type BgMode = "image" | "solid" | "gradient";
export type BgFit = "cover" | "contain" | "stretch";
export type MaterialKind = "none" | "solid" | "glass" | "acrylic" | "mica";
export type Position =
  | "center"
  | "topLeft"
  | "topRight"
  | "bottomLeft"
  | "bottomRight"
  | "custom";

export interface UiConfig {
  source: {
    mode: SourceMode;
    inputTarget: string;
    apiKey: string;
    refreshIntervalMinutes: number;
  };
  background: {
    mode: BgMode;
    imagePath: string;
    fit: BgFit;
    colorPrimary: string;
    colorSecondary: string;
    /** Image framing: zoom around the centre, 0.5 - 4.0. */
    zoom: number;
    /** Horizontal offset of the framing, -1 (left) to 1 (right). */
    panX: number;
    /** Vertical offset of the framing, -1 (up) to 1 (down). */
    panY: number;
  };
  card: {
    position: Position;
    offsetX: number;
    offsetY: number;
    material: MaterialKind;
    tintColor: string;
    opacity: number;
    blurRadius: number;
    cornerRadius: number;
  };
  track: {
    boxSize: number;
    boxGap: number;
    boxRadius: number;
    colors: [string, string, string, string, string];
    showStatsHeader: boolean;
    textColor: string;
  };
  startWithWindows: boolean;
}

export interface AppStatus {
  daemonRunning: boolean;
  autostart: boolean;
  version: string;
  resolution: [number, number];
  configPath: string;
  cachePath: string;
  mica: boolean;
}

export interface Bootstrap {
  config: UiConfig;
  status: AppStatus;
}

export interface ConnectionStats {
  total: number;
  bestDay: number;
  streak: number;
  days: number;
}

export interface ApplyResult {
  message: string;
  daemonRunning: boolean;
  autostart: boolean;
}

/** Immutable config update: the recipe receives the current config and returns a new one. */
export type UpdateFn = (recipe: (current: UiConfig) => UiConfig) => void;

export type PageId = "source" | "background" | "material" | "track";

export type ToastKind = "success" | "error" | "info";

export interface ToastData {
  id: number;
  kind: ToastKind;
  text: string;
}

/** Transparent card layer the live preview composites over a moved background. */
export interface CardOverlay {
  /** PNG data URL: card + heatmap, transparent elsewhere. */
  png: string;
  /** Card rect in canvas pixels: x, y, w, h. */
  card: [number, number, number, number];
  radius: number;
  /** Backdrop blur in canvas pixels, material factor already applied. */
  blur: number;
  canvas: [number, number];
}
