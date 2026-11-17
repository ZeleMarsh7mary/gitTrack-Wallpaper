import { Spinner } from "./controls";
import { Icon } from "./Icons";

export function Preview({
  src,
  updating,
  error,
  resolution,
}: {
  src: string | null;
  updating: boolean;
  error: string | null;
  resolution: [number, number];
}) {
  return (
    <div className="preview">
      <div
        className={`preview-frame${updating ? " is-updating" : ""}`}
        aria-busy={updating}
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
      </div>
      <p className="preview-caption">
        {resolution[0]} x {resolution[1]} <span aria-hidden="true">·</span> live render
      </p>
    </div>
  );
}
