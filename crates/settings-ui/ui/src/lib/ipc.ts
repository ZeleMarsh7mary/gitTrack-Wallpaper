import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_CONFIG } from "./defaults";
import { demoBackground, demoCardOverlay, demoPreview } from "./demo";
import type {
  AppStatus,
  ApplyResult,
  Bootstrap,
  CardOverlay,
  ConnectionStats,
  UiConfig,
} from "./types";

/**
 * Thin IPC layer. Inside Tauri every call goes through the real command; in a
 * plain browser (design review, `npm run dev`) a small demo backend answers so
 * the page renders without the Rust side.
 */
export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

const DEMO_STATUS: AppStatus = {
  daemonRunning: true,
  autostart: false,
  version: "0.1.0",
  resolution: [1920, 1080],
  configPath: "C:\\Users\\you\\AppData\\Roaming\\GitTrackWallpaper\\config.toml",
  cachePath: "C:\\Users\\you\\AppData\\Local\\GitTrackWallpaper\\cache.json",
  mica: false,
};

const wait = (ms: number) =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });

async function demoCall<T>(
  command: string,
  args: Record<string, unknown> | undefined,
): Promise<T> {
  switch (command) {
    case "bootstrap":
      await wait(240);
      return {
        config: DEFAULT_CONFIG,
        status: DEMO_STATUS,
      } as T;
    case "render_preview":
      await wait(280);
      return demoPreview(
        (args?.config as UiConfig | undefined) ?? DEFAULT_CONFIG,
      ) as T;
    case "test_connection":
      await wait(600);
      return { total: 1284, bestDay: 27, streak: 12, days: 371 } as T;
    case "save_and_apply": {
      await wait(400);
      const config = args?.config as UiConfig | undefined;
      return {
        message: "Settings saved. Wallpaper updated for your desktop.",
        daemonRunning: true,
        autostart: Boolean(config?.startWithWindows ?? false),
      } as T;
    }
    case "pick_image_file":
      await wait(120);
      return "C:\\Users\\Public\\Pictures\\wallpaper.png" as T;
    case "show_window":
      return false as T;
    case "background_image":
      return (args?.path ? demoBackground() : null) as T;
    case "render_card_overlay":
      return {
        png: demoCardOverlay(),
        card: [300, 168, 360, 204],
        radius: 10,
        blur: 12,
        canvas: [960, 540],
      } as T;
    default:
      throw `Unknown command: ${command}`;
  }
}

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (isTauri) return invoke<T>(command, args);
  return demoCall<T>(command, args);
}

export const bootstrap = () => call<Bootstrap>("bootstrap");

export const renderPreview = (config: UiConfig) =>
  call<string>("render_preview", { config });

export const testConnection = (config: UiConfig) =>
  call<ConnectionStats>("test_connection", { config });

export const saveAndApply = (config: UiConfig) =>
  call<ApplyResult>("save_and_apply", { config });

export const pickImageFile = () => call<string | null>("pick_image_file");

export const showWindow = () => call<boolean>("show_window");

/** Raw background picture for the live composite; decoded and cached by Rust. */
export const backgroundImage = (path: string) =>
  call<string | null>("background_image", { path });

/** Card + heatmap on a transparent canvas, for the live composite. */
export const renderCardOverlay = (config: UiConfig) =>
  call<CardOverlay>("render_card_overlay", { config });

// Without Mica the window must paint the flat WinUI dark background. Doing it
// before the first frame avoids a white flash in the browser demo.
if (!isTauri && typeof document !== "undefined") {
  document.documentElement.classList.add("no-mica");
}
