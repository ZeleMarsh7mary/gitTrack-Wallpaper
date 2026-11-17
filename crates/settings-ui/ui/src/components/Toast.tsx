import { Icon } from "./Icons";
import type { ToastData } from "../lib/types";

const ICONS = {
  success: "checkCircle",
  error: "warningCircle",
  info: "infoCircle",
} as const;

export function Toast({ toast }: { toast: ToastData | null }) {
  if (!toast) return null;

  return (
    <div
      key={toast.id}
      className={`toast toast-${toast.kind}`}
      role={toast.kind === "error" ? "alert" : "status"}
    >
      <span className="toast-icon">
        <Icon name={ICONS[toast.kind]} />
      </span>
      <span className="toast-text">{toast.text}</span>
    </div>
  );
}
