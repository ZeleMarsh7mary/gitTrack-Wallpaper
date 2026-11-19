import { useEffect, useRef } from "react";
import { Spinner } from "./controls";
import { Icon } from "./Icons";

export type GridMode = "off" | "thirds" | "golden";

/** Framing controls, only wired up on the Background page in Image mode. */
export interface Framing {
  grid: GridMode;
  zoom: number;
  panX: number;
  panY: number;
  onGridChange: (mode: GridMode) => void;
  onPan: (panX: number, panY: number) => void;
  onZoom: (zoom: number) => void;
  onReset: () => void;
  onInteract: (active: boolean) => void;
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

export function Preview({
  src,
  updating,
  error,
  resolution,
  framing,
}: {
  src: string | null;
  updating: boolean;
  error: string | null;
  resolution: [number, number];
  framing: Framing | null;
}) {
  const frameRef = useRef<HTMLDivElement>(null);
  const framingRef = useRef<Framing | null>(framing);
  framingRef.current = framing;
  const dragRef = useRef<{
    x: number;
    y: number;
    panX: number;
    panY: number;
    width: number;
    height: number;
  } | null>(null);

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
      active.onZoom(Number(clamp(active.zoom * factor, ZOOM_MIN, ZOOM_MAX).toFixed(3)));
    };
    element.addEventListener("wheel", onWheel, { passive: false });
    return () => element.removeEventListener("wheel", onWheel);
  }, []);

  const onPointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
    const active = framing;
    if (!active || event.button !== 0) return;
    const rect = event.currentTarget.getBoundingClientRect();
    dragRef.current = {
      x: event.clientX,
      y: event.clientY,
      panX: active.panX,
      panY: active.panY,
      width: rect.width,
      height: rect.height,
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
    // Half a frame of travel covers the whole pan range, so the picture follows
    // the cursor 1:1 while it fills the frame.
    const panX = clamp(drag.panX + (event.clientX - drag.x) / (drag.width * 0.5), -1, 1);
    const panY = clamp(drag.panY + (event.clientY - drag.y) / (drag.height * 0.5), -1, 1);
    active.onPan(Number(panX.toFixed(3)), Number(panY.toFixed(3)));
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
        {src ? (
          <img
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
