import type { ReactElement } from "react";

export type IconName =
  | "branch"
  | "user"
  | "key"
  | "clock"
  | "image"
  | "folder"
  | "frame"
  | "droplet"
  | "layers"
  | "circleHalf"
  | "blur"
  | "corner"
  | "square"
  | "roundedSquare"
  | "gridGap"
  | "palette"
  | "text"
  | "type"
  | "windows"
  | "move"
  | "arrowX"
  | "arrowY"
  | "eye"
  | "eyeOff"
  | "chevron"
  | "checkCircle"
  | "warningCircle"
  | "infoCircle";

const GLYPHS: Record<IconName, ReactElement> = {
  branch: (
    <>
      <circle cx="4.9" cy="4.2" r="1.6" />
      <circle cx="11.1" cy="4.2" r="1.6" />
      <circle cx="4.9" cy="11.8" r="1.6" />
      <path d="M4.9 5.8v4.4" />
      <path d="M11.1 5.8v1.1c0 1.7-1.4 2.4-3.4 2.6" />
    </>
  ),
  user: (
    <>
      <circle cx="8" cy="5.4" r="2.4" />
      <path d="M3.4 13c.5-2.3 2.3-3.6 4.6-3.6s4.1 1.3 4.6 3.6" />
    </>
  ),
  key: (
    <>
      <circle cx="5.8" cy="10.2" r="2.6" />
      <path d="M7.7 8.3 13 3M10.8 5.2l1.6 1.6M9.2 6.8l1.4 1.4" />
    </>
  ),
  clock: (
    <>
      <circle cx="8" cy="8" r="5.4" />
      <path d="M8 4.8V8l2.2 1.4" />
    </>
  ),
  image: (
    <>
      <rect x="2.6" y="3.4" width="10.8" height="9.2" rx="1.6" />
      <circle cx="6.1" cy="6.5" r="1" />
      <path d="M3.2 11.4 6.6 8.5l2 1.7 1.9-1.6 2.2 2.1" />
    </>
  ),
  folder: (
    <path d="M2.5 5.2c0-.9.7-1.6 1.6-1.6h2.3l1.3 1.4h4.2c.9 0 1.6.7 1.6 1.6v4.9c0 .9-.7 1.6-1.6 1.6H4.1c-.9 0-1.6-.7-1.6-1.6z" />
  ),
  frame: <path d="M5.6 2.5v10.9M10.4 2.5v10.9M2.5 5.6h10.9M2.5 10.4h10.9" />,
  droplet: (
    <path d="M8 2.6c2 2.4 3.4 4.2 3.4 6A3.4 3.4 0 0 1 8 12a3.4 3.4 0 0 1-3.4-3.4c0-1.8 1.4-3.6 3.4-6Z" />
  ),
  layers: (
    <>
      <path d="M8 2.6 14 6l-6 3.4L2 6z" />
      <path d="M2.7 9.1 8 12.1l5.3-3" />
    </>
  ),
  circleHalf: (
    <>
      <circle cx="8" cy="8" r="5.4" />
      <path d="M8 2.6a5.4 5.4 0 0 1 0 10.8z" fill="currentColor" stroke="none" />
    </>
  ),
  blur: (
    <>
      <circle cx="8" cy="8" r="5.4" strokeDasharray="0.1 2.3" />
      <circle cx="8" cy="8" r="2.3" />
    </>
  ),
  corner: <path d="M13.2 3H8.6A5.6 5.6 0 0 0 3 8.6v4.6" />,
  square: <rect x="3.5" y="3.5" width="9" height="9" rx="1" />,
  roundedSquare: <rect x="3.5" y="3.5" width="9" height="9" rx="3.2" />,
  gridGap: (
    <>
      <rect x="2.5" y="2.5" width="4.6" height="4.6" rx="1" />
      <rect x="8.9" y="2.5" width="4.6" height="4.6" rx="1" />
      <rect x="2.5" y="8.9" width="4.6" height="4.6" rx="1" />
      <rect x="8.9" y="8.9" width="4.6" height="4.6" rx="1" />
    </>
  ),
  palette: (
    <>
      <rect x="2.5" y="2.5" width="4.6" height="4.6" rx="1.2" />
      <rect x="8.9" y="2.5" width="4.6" height="4.6" rx="1.2" />
      <rect x="2.5" y="8.9" width="4.6" height="4.6" rx="1.2" />
      <path d="M11.2 9.1v4.4M9 11.3h4.4" />
    </>
  ),
  text: <path d="M3.4 4.6h9.2M3.4 8h9.2M3.4 11.4h5.6" />,
  type: <path d="M3.5 12.5 8 3.5l4.5 9M5.3 9.4h5.4" />,
  windows: (
    <path d="M2.6 4.2 7 3.6v4.1H2.6zM8.4 3.4l4.9-.7v5H8.4zM2.6 8.6H7v4.1l-4.4-.6zM8.4 8.6h4.9v5l-4.9-.7z" />
  ),
  move: (
    <>
      <path d="M8 2.5v11M2.5 8h11" />
      <circle cx="8" cy="8" r="2.6" />
    </>
  ),
  arrowX: <path d="M2.5 8h11M6 4.5 2.5 8 6 11.5M10 4.5 13.5 8 10 11.5" />,
  arrowY: <path d="M8 2.5v11M4.5 6 8 2.5 11.5 6M4.5 10 8 13.5 11.5 10" />,
  eye: (
    <>
      <path d="M1.9 8S4.1 4.2 8 4.2 14.1 8 14.1 8 11.9 11.8 8 11.8 1.9 8 1.9 8Z" />
      <circle cx="8" cy="8" r="1.9" />
    </>
  ),
  eyeOff: (
    <>
      <path d="M3 6.3C2.3 7 1.9 8 1.9 8S4.1 11.8 8 11.8c1.1 0 2-.3 2.8-.8M12.4 10.1c1.1-.9 1.7-2.1 1.7-2.1S11.9 4.2 8 4.2c-.6 0-1.1.1-1.6.2" />
      <path d="M6.6 6.6 9.4 9.4M2 2l12 12" />
    </>
  ),
  chevron: <path d="M4 6.2 8 10l4-3.8" />,
  checkCircle: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M5 8.2 7.1 10.3 11.1 5.9" />
    </>
  ),
  warningCircle: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 4.7v4.2" />
      <circle cx="8" cy="11.2" r="0.7" fill="currentColor" stroke="none" />
    </>
  ),
  infoCircle: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 7.2v3.6" />
      <circle cx="8" cy="5.1" r="0.7" fill="currentColor" stroke="none" />
    </>
  ),
};

export function Icon({
  name,
  size = 16,
  className,
}: {
  name: IconName;
  size?: number;
  className?: string;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.25}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      {GLYPHS[name]}
    </svg>
  );
}

/** Product mark: a tiny heatmap tile in GitHub greens. */
export function ProductMark() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
      <rect x="0.5" y="0.5" width="15" height="15" rx="4" fill="#2EA043" fillOpacity="0.14" />
      <rect x="0.5" y="0.5" width="15" height="15" rx="4" fill="none" stroke="#2EA043" strokeOpacity="0.45" />
      <rect x="4" y="4" width="3" height="3" rx="0.8" fill="#0e4429" />
      <rect x="9" y="4" width="3" height="3" rx="0.8" fill="#26a641" />
      <rect x="4" y="9" width="3" height="3" rx="0.8" fill="#26a641" />
      <rect x="9" y="9" width="3" height="3" rx="0.8" fill="#39d353" />
    </svg>
  );
}
