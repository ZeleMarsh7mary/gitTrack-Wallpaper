import {
  Button,
  ColorSwatch,
  Section,
  Segmented,
  SettingRow,
  Select,
  TextField,
} from "../components/controls";
import type { UiConfig, UpdateFn } from "../lib/types";

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

      {background.mode === "image" ? (
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

      {background.mode === "image" ? (
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
                },
              }))
            }
          />
        </SettingRow>
      ) : null}

      {background.mode !== "image" ? (
        <SettingRow
          icon="droplet"
          title="Primary colour"
          subtitle={
            background.mode === "solid"
              ? "The flat colour that fills the desktop."
              : "Top-left end of the gradient."
          }
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
      ) : null}

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
