import { useEffect, useRef, useState } from "react";
import { Spinner } from "./controls";
import { Icon } from "./Icons";
import type { BgFit, CardOverlay } from "../lib/types";

export type GridMode = "off" | "thirds" | "golden";

/** Framing controls, only wired up on the Background page in Image mode. */
export interface Framing {
  grid: GridMode;
  zoom: number;
  panX: number;
  panY: number;
  /** Lowest zoom the current fit allows (1.0 in Cover, so no gap can appear). */
  minZoom: number;
  onGridChange: (mode: GridMode) => void;
  onPan: (panX: number, panY: number) => void;
  onZoom: (zoom: number) => void;
  onReset: () => void;
  onInteract: (active: boolean) => void;
}

/** What the live composite needs to move the picture without asking Rust. */
export interface LiveFraming {
  image: string | null;
  card: CardOverlay | null;
  fit: BgFit;
}

export const ZOOM_MIN = 0.5;
export const ZOOM_MAX = 4;

/** Line positions in percent; two lines each way gives the 9-box composition. */
const GRID_LINES: Record<Exclude<GridMode, "off">, readonly [number, number]> = {
  thirds: [33.333, 66.667],
  golden: [38.2, 61.8],
};

const GRID_OPTIONS: ReadonlyArray<{ value: GridMode; label: string }> = [
  { value: "off", label: "Off" },
  { value: "thirds", label: "Thirds" },
  { value: "golden", label: "Golden" },
];

const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

/** Lowest zoom that still fills the frame, mirroring `renderer/background.rs`. */
export const minZoomFor = (fit: BgFit) => (fit === "cover" ? 1 : ZOOM_MIN);

interface Geometry {
  left: number;
  top: number;
  width: number;
  height: number;
  slackX: number;
  slackY: number;
}

/**
 * Pixel geometry for the live layer. Identical to `renderer::background`: fit,
 * then zoom (never below 1.0 in Cover), then pan across the slack the picture
 * leaves — so an edge can never expose the backdrop.
 */
function geometry(
  canvas: { w: number; h: number },
  natural: { w: number; h: number },
  fit: BgFit,
  zoom: number,
  panX: number,
  panY: number,
): Geometry {
  const sx = canvas.w / natural.w;
  const sy = canvas.h / natural.h;
  let baseX = sx;
  let baseY = sy;
  if (fit === "cover") {
    baseX = Math.max(sx, sy);
    baseY = baseX;
  } else if (fit === "contain") {
    baseX = Math.min(sx, sy);
    baseY = baseX;
  }
  const effective = fit === "cover" ? Math.max(zoom, 1) : zoom;
  const width = natural.w * baseX * effective;
  const height = natural.h * baseY * effective;
  const slackX = Math.max((width - canvas.w) / 2, 0);
  const slackY = Math.max((height - canvas.h) / 2, 0);
  const centreX = canvas.w / 2 + clamp(panX, -1, 1) * slackX;
  const centreY = canvas.h / 2 + clamp(panY, -1, 1) * slackY;
  return {
    left: centreX - width / 2,
    top: centreY - height / 2,
    width,
    height,
    slackX,
    slackY,
  };
}

export function Preview({
  src,
  updating,
  error,
  resolution,
  framing,
  live,
  interacting,
}: {
  src: string | null;
  updating: boolean;
  error: string | null;
  resolution: [number, number];
  framing: Framing | null;
  live: LiveFraming | null;
  /** True while a drag or a wheel gesture is running. */
  interacting: boolean;
}) {
  const frameRef = useRef<HTMLDivElement>(null);
  const framingRef = useRef<Framing | null>(framing);
  framingRef.current = framing;
  const dragRef = useRef<{
    x: number;
    y: number;
    panX: number;
    panY: number;
    slackX: number;
    slackY: number;
  } | null>(null);
  const wheelIdle = useRef<number | null>(null);
  const [natural, setNatural] = useState<{ w: number; h: number } | null>(null);
  const [canvasSize, setCanvasSize] = useState<{ w: number; h: number } | null>(null);

  // Decode the background once so the first drag frame already has geometry.
  useEffect(() => {
    const url = live?.image;
    if (!url) {
      setNatural(null);
      return;
    }
    const probe = new Image();
    let cancelled = false;
    probe.onload = () => {
      if (!cancelled && probe.naturalWidth > 0) {
        setNatural({ w: probe.naturalWidth, h: probe.naturalHeight });
      }
    };
    probe.src = url;
    return () => {
      cancelled = true;
    };
  }, [live?.image]);

  // The canvas size in CSS pixels is the frame size; the transform depends on it.
  useEffect(() => {
    const element = frameRef.current;
    if (!element) return;
    const measure = () => setCanvasSize({ w: element.clientWidth, h: element.clientHeight });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  // React registers wheel listeners as passive, so the zoom handler has to be
  // native to call preventDefault() and keep the page from scrolling.
  useEffect(() => {
    const element = frameRef.current;
    if (!element) return;
    const onWheel = (event: WheelEvent) => {
      const active = framingRef.current;
      if (!active || event.deltaY === 0) return;
      event.preventDefault();
      const factor = event.deltaY < 0 ? 1.06 : 1 / 1.06;
      const min = active.minZoom ?? ZOOM_MIN;
      active.onZoom(Number(clamp(active.zoom * factor, min, ZOOM_MAX).toFixed(3)));
      // The gesture ends when the wheel stops, not on a pointerup.
      active.onInteract(true);
      if (wheelIdle.current !== null) window.clearTimeout(wheelIdle.current);
      wheelIdle.current = window.setTimeout(() => {
        wheelIdle.current = null;
        framingRef.current?.onInteract(false);
      }, 180);
    };
    element.addEventListener("wheel", onWheel, { passive: false });
    return () => {
      element.removeEventListener("wheel", onWheel);
      if (wheelIdle.current !== null) window.clearTimeout(wheelIdle.current);
    };
  }, []);

  const currentGeometry = (): Geometry | null => {
    const active = framing;
    const frame = frameRef.current;
    if (!active || !frame || !natural) return null;
    const size = canvasSize ?? { w: frame.clientWidth, h: frame.clientHeight };
    if (size.w <= 0 || size.h <= 0) return null;
    return geometry(size, natural, live?.fit ?? "cover", active.zoom, active.panX, active.panY);
  };

  const onPointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
    const active = framing;
    if (!active || event.button !== 0) return;
    const geo = currentGeometry();
    if (!geo) return;
    dragRef.current = {
      x: event.clientX,
      y: event.clientY,
      panX: active.panX,
      panY: active.panY,
      slackX: geo.slackX,
      slackY: geo.slackY,
    };
    // Capture keeps the drag alive when the cursor leaves the frame. It throws
    // for a pointer that is no longer active, which must not break the drag.
    try {
      event.currentTarget.setPointerCapture(event.pointerId);
    } catch {
      /* not capturable, the pointerup still ends the gesture */
    }
    active.onInteract(true);
  };

  const onPointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current;
    const active = framing;
    if (!drag || !active) return;
    const dx = event.clientX - drag.x;
    const dy = event.clientY - drag.y;
    // A pan of 1.0 is exactly one slack, so the picture sticks to the cursor.
    const nextX =
      drag.slackX > 0.5 ? clamp((drag.panX * drag.slackX + dx) / drag.slackX, -1, 1) : 0;
    const nextY =
      drag.slackY > 0.5 ? clamp((drag.panY * drag.slackY + dy) / drag.slackY, -1, 1) : 0;
    active.onPan(Number(nextX.toFixed(4)), Number(nextY.toFixed(4)));
  };

  const endDrag = (event: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    dragRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    framing?.onInteract(false);
  };

  const gridLines = framing && framing.grid !== "off" ? GRID_LINES[framing.grid] : null;
  const overlay = live?.card ?? null;
  const geo = framing ? currentGeometry() : null;
  // The composite only stands in for the real render while a gesture runs; after
  // the pointer is released the authoritative tiny-skia output takes over again.
  const showLive = Boolean(interacting && framing && live?.image && geo);
  const canvasWidth = overlay?.canvas[0] ?? 0;
  const toCss = canvasSize && canvasWidth > 0 ? canvasSize.w / canvasWidth : 0;
  const blurPx = overlay && toCss > 0 ? overlay.blur * toCss : 0;

  return (
    <div className="preview">
      <div
        ref={frameRef}
        className={`preview-frame${updating ? " is-updating" : ""}${framing ? " is-interactive" : ""}`}
        aria-busy={updating}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        title={framing ? "Drag to pan · scroll to zoom" : undefined}
      >
        {/* Live composite: the picture is moved by CSS while the gesture runs, so
            no render is requested until it ends. */}
        {showLive && geo && live?.image ? (
          <div className="preview-live" aria-hidden="true">
            <img
              className="preview-live-image"
              src={live.image}
              alt=""
              draggable={false}
              style={{
                width: `${geo.width}px`,
                height: `${geo.height}px`,
                transform: `translate3d(${geo.left}px, ${geo.top}px, 0)`,
              }}
            />
            {overlay && blurPx > 0.2 ? (
              <div
                className="preview-live-blur"
                style={{
                  left: `${overlay.card[0] * toCss}px`,
                  top: `${overlay.card[1] * toCss}px`,
                  width: `${overlay.card[2] * toCss}px`,
                  height: `${overlay.card[3] * toCss}px`,
                  borderRadius: `${overlay.radius * toCss}px`,
                  backdropFilter: `blur(${blurPx.toFixed(2)}px)`,
                }}
              />
            ) : null}
            {overlay ? (
              <img className="preview-live-card" src={overlay.png} alt="" draggable={false} />
            ) : null}
          </div>
        ) : null}

        {src ? (
          <img
            className={`preview-shot${showLive ? " is-hidden" : ""}`}
            src={src}
            alt="Live render of the wallpaper with the contribution card."
            draggable={false}
          />
        ) : (
          <div className={`preview-empty${error ? " is-error" : ""}`}>
            {error ? (
              <>
                <Icon name="warningCircle" />
                <span>Preview unavailable</span>
              </>
            ) : (
              <>
                <Spinner />
                <span>Rendering preview…</span>
              </>
            )}
          </div>
        )}

        {/* Composition guides: DOM only, the renderer never draws them. */}
        {gridLines ? (
          <div className="grid-layer" aria-hidden="true">
            {gridLines.map((line) => (
              <span
                key={`v${line}`}
                className="grid-line is-vertical"
                style={{ left: `${line}%` }}
              />
            ))}
            {gridLines.map((line) => (
              <span
                key={`h${line}`}
                className="grid-line is-horizontal"
                style={{ top: `${line}%` }}
              />
            ))}
            {gridLines.flatMap((x) =>
              gridLines.map((y) => (
                <span key={`d${x}-${y}`} className="grid-dot" style={{ left: `${x}%`, top: `${y}%` }} />
              )),
            )}
          </div>
        ) : null}
      </div>

      <div className="preview-meta">
        <p className="preview-caption">
          {resolution[0]} x {resolution[1]} <span aria-hidden="true">·</span> live render
        </p>
        {framing ? (
          <div className="preview-tools">
            <span className="preview-tools-label" id="preview-grid-label">
              Grid
            </span>
            <div className="preview-tools-group" role="radiogroup" aria-labelledby="preview-grid-label">
              {GRID_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  role="radio"
                  aria-checked={framing.grid === option.value}
                  className={`preview-tool${framing.grid === option.value ? " is-active" : ""}`}
                  onClick={() => framing.onGridChange(option.value)}
                >
                  {option.label}
                </button>
              ))}
            </div>
            <button
              type="button"
              className="link-button"
              onClick={framing.onReset}
              title="Zoom 1.00x, X 0%, Y 0%"
            >
              Reset framing
            </button>
          </div>
        ) : null}
      </div>
    </div>
  );
}
