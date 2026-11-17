import { Icon, ProductMark } from "./Icons";
import type { IconName } from "./Icons";
import type { AppStatus, PageId } from "../lib/types";

const ITEMS: ReadonlyArray<{ id: PageId; label: string; icon: IconName }> = [
  { id: "source", label: "Git Source", icon: "branch" },
  { id: "background", label: "Background", icon: "image" },
  { id: "material", label: "Material & Blur", icon: "layers" },
  { id: "track", label: "Track & Boxes", icon: "gridGap" },
];

export function NavRail({
  page,
  onSelect,
  status,
}: {
  page: PageId;
  onSelect: (page: PageId) => void;
  status: AppStatus | null;
}) {
  return (
    <nav className="rail" aria-label="Settings sections">
      <div className="rail-product">
        <ProductMark />
        <span className="rail-product-name">GitTrack Wallpaper</span>
      </div>

      <div className="rail-nav">
        {ITEMS.map((item) => (
          <button
            key={item.id}
            type="button"
            className="nav-item"
            aria-current={page === item.id ? "page" : undefined}
            onClick={() => onSelect(item.id)}
          >
            <Icon name={item.icon} />
            <span>{item.label}</span>
          </button>
        ))}
      </div>

      <div className="rail-footer">
        <span className={`badge${status?.daemonRunning ? " is-on" : ""}`}>
          <span className="dot" />
          Daemon: {status ? (status.daemonRunning ? "running" : "stopped") : "unknown"}
        </span>
        <span className={`badge${status?.autostart ? " is-on" : ""}`}>
          <span className="dot" />
          Auto-start: {status ? (status.autostart ? "on" : "off") : "unknown"}
        </span>
        <span className="rail-version">v{status?.version ?? "0.1.0"}</span>
      </div>
    </nav>
  );
}
