import {
  Button,
  ColorSwatch,
  Section,
  Segmented,
  SettingRow,
  Select,
  Slider,
  TextField,
} from "../components/controls";
import type { UiConfig, UpdateFn } from "../lib/types";

/** Pan is stored normalized; the badge shows it the way the sliders describe it. */
const formatPercent = (value: number) => `${value > 0 ? "+" : ""}${Math.round(value * 100)}%`;

const PRIMARY_SUBTITLE: Record<UiConfig["background"]["mode"], string> = {
  image: "Fills whatever the picture leaves uncovered.",
  solid: "The flat colour that fills the desktop.",
  gradient: "Top-left end of the gradient.",
};

export function BackgroundPage({
  config,
  update,
  onBrowse,
  browseBusy,
  actionsDisabled,
}: {
  config: UiConfig;
  update: UpdateFn;
  onBrowse: () => void;
  browseBusy: boolean;
  actionsDisabled: boolean;
}) {
  const { background } = config;
  const isImage = background.mode === "image";

  return (
    <Section title="Wallpaper">
      <SettingRow
        icon="image"
        title="Mode"
        subtitle="Draw behind the card with an image, a flat colour or a gradient."
      >
        <Segmented
          ariaLabel="Wallpaper mode"
          value={background.mode}
          options={[
            { value: "image", label: "Image" },
            { value: "solid", label: "Solid" },
            { value: "gradient", label: "Gradient" },
          ]}
          onChange={(mode) =>
            update((current) => ({
              ...current,
              background: { ...current.background, mode },
            }))
          }
        />
      </SettingRow>

      {isImage ? (
        <SettingRow
          stacked
          icon="folder"
          title="Image file"
          subtitle="The picture is scaled to the screen and re-used on every render."
        >
          <div className="file-control">
            <TextField
              readOnly
              value={background.imagePath}
              placeholder="No image selected"
              ariaLabel="Background image file"
              onChange={() => undefined}
            />
            <Button
              variant="secondary"
              onClick={onBrowse}
              busy={browseBusy}
              busyLabel="Browsing…"
              disabled={actionsDisabled}
            >
              Browse…
            </Button>
          </div>
        </SettingRow>
      ) : null}

      {isImage ? (
        <SettingRow
          icon="frame"
          title="Fit"
          subtitle="How the picture fills the screen."
        >
          <Select
            ariaLabel="Image fit"
            value={background.fit}
            options={[
              { value: "cover", label: "Cover" },
              { value: "contain", label: "Contain" },
              { value: "stretch", label: "Stretch" },
            ]}
            onChange={(fit) =>
              update((current) => ({
                ...current,
                background: {
                  ...current.background,
                  fit: fit as UiConfig["background"]["fit"],
                  // Cover cannot go below 1.0, so do not keep a value the renderer
                  // would only clamp silently.
                  zoom:
                    fit === "cover"
                      ? Math.max(current.background.zoom, 1)
                      : current.background.zoom,
                },
              }))
            }
          />
        </SettingRow>
      ) : null}

      {isImage ? (
        <SettingRow
          icon="frame"
          title="Zoom"
          subtitle="Scales the picture around the screen centre. Scroll over the preview to zoom."
        >
          <Slider
            ariaLabel="Wallpaper zoom"
            value={background.zoom}
            min={background.fit === "cover" ? 1 : 0.5}
            max={4}
            step={0.05}
            format={(value) => `${value.toFixed(2)}x`}
            onChange={(zoom) =>
              update((current) => ({ ...current, background: { ...current.background, zoom } }))
            }
          />
        </SettingRow>
      ) : null}

      {isImage ? (
        <SettingRow
          icon="arrowX"
          title="Position X"
          subtitle="Move the picture left and right, or drag inside the preview."
        >
          <Slider
            ariaLabel="Wallpaper horizontal position"
            value={background.panX}
            min={-1}
            max={1}
            step={0.01}
            format={formatPercent}
            onChange={(panX) =>
              update((current) => ({ ...current, background: { ...current.background, panX } }))
            }
          />
        </SettingRow>
      ) : null}

      {isImage ? (
        <SettingRow
          icon="arrowY"
          title="Position Y"
          subtitle="Move the picture up and down, or drag inside the preview."
        >
          <Slider
            ariaLabel="Wallpaper vertical position"
            value={background.panY}
            min={-1}
            max={1}
            step={0.01}
            format={formatPercent}
            onChange={(panY) =>
              update((current) => ({ ...current, background: { ...current.background, panY } }))
            }
          />
        </SettingRow>
      ) : null}

      <SettingRow
        icon="droplet"
        title="Primary colour"
        subtitle={PRIMARY_SUBTITLE[background.mode]}
      >
        <ColorSwatch
          label="Primary colour"
          value={background.colorPrimary}
          onChange={(colorPrimary) =>
            update((current) => ({
              ...current,
              background: { ...current.background, colorPrimary },
            }))
          }
        />
      </SettingRow>

      {background.mode === "gradient" ? (
        <SettingRow
          icon="droplet"
          title="Secondary colour"
          subtitle="Bottom-right end of the gradient."
        >
          <ColorSwatch
            label="Secondary colour"
            value={background.colorSecondary}
            onChange={(colorSecondary) =>
              update((current) => ({
                ...current,
                background: { ...current.background, colorSecondary },
              }))
            }
          />
        </SettingRow>
      ) : null}
    </Section>
  );
}
