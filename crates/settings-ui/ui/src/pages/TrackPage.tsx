import {
  ColorSwatch,
  PalettePicker,
  Section,
  SettingRow,
  Slider,
  Switch,
} from "../components/controls";
import type { UiConfig, UpdateFn } from "../lib/types";

export function TrackPage({
  config,
  update,
}: {
  config: UiConfig;
  update: UpdateFn;
}) {
  const { track } = config;
  const radiusMax = Math.max(track.boxSize / 2, 0.5);

  return (
    <>
      <Section title="Heatmap">
        <SettingRow
          icon="square"
          title="Box size"
          subtitle="Edge length of each contribution square."
        >
          <Slider
            ariaLabel="Box size"
            value={track.boxSize}
            min={6}
            max={40}
            step={1}
            format={(value) => `${Math.round(value)} px`}
            onChange={(boxSize) =>
              update((current) => ({
                ...current,
                track: {
                  ...current.track,
                  boxSize: Math.round(boxSize),
                  boxRadius: Math.min(
                    current.track.boxRadius,
                    Math.round(boxSize) / 2,
                  ),
                },
              }))
            }
          />
        </SettingRow>

        <SettingRow
          icon="gridGap"
          title="Box gap"
          subtitle="Space between the contribution squares."
        >
          <Slider
            ariaLabel="Box gap"
            value={track.boxGap}
            min={0}
            max={12}
            step={1}
            format={(value) => `${Math.round(value)} px`}
            onChange={(boxGap) =>
              update((current) => ({
                ...current,
                track: { ...current.track, boxGap: Math.round(boxGap) },
              }))
            }
          />
        </SettingRow>

        <SettingRow
          icon="roundedSquare"
          title="Box radius"
          subtitle="Corner rounding on each square, up to half its size."
        >
          <Slider
            ariaLabel="Box corner radius"
            value={Math.min(track.boxRadius, radiusMax)}
            min={0}
            max={radiusMax}
            step={0.5}
            format={(value) => `${value.toFixed(1)} px`}
            onChange={(boxRadius) =>
              update((current) => ({
                ...current,
                track: {
                  ...current.track,
                  boxRadius: Math.min(
                    Math.round(boxRadius * 2) / 2,
                    current.track.boxSize / 2,
                  ),
                },
              }))
            }
          />
        </SettingRow>

        <SettingRow
          icon="palette"
          title="Colour ramp"
          subtitle="Five levels, from quiet days to your best one."
        >
          <PalettePicker
            colors={track.colors}
            onColorChange={(index, hex) =>
              update((current) => {
                const colors = [...current.track.colors] as [
                  string,
                  string,
                  string,
                  string,
                  string,
                ];
                colors[index] = hex;
                return { ...current, track: { ...current.track, colors } };
              })
            }
            onPreset={(colors) =>
              update((current) => ({
                ...current,
                track: { ...current.track, colors: [...colors] as UiConfig["track"]["colors"] },
              }))
            }
          />
        </SettingRow>

        <SettingRow
          icon="text"
          title="Show stats header"
          subtitle="Print total contributions and streak above the grid."
        >
          <Switch
            ariaLabel="Show stats header"
            checked={track.showStatsHeader}
            onChange={(showStatsHeader) =>
              update((current) => ({
                ...current,
                track: { ...current.track, showStatsHeader },
              }))
            }
          />
        </SettingRow>

        <SettingRow
          icon="type"
          title="Text colour"
          subtitle="Used by the stats header inside the card."
        >
          <ColorSwatch
            label="Card text colour"
            value={track.textColor}
            onChange={(textColor) =>
              update((current) => ({
                ...current,
                track: { ...current.track, textColor },
              }))
            }
          />
        </SettingRow>
      </Section>

      <Section title="System">
        <SettingRow
          icon="windows"
          title="Start with Windows"
          subtitle="Launch GitWallpaper when you sign in."
        >
          <Switch
            ariaLabel="Start with Windows"
            checked={config.startWithWindows}
            onChange={(startWithWindows) =>
              update((current) => ({ ...current, startWithWindows }))
            }
          />
        </SettingRow>
      </Section>
    </>
  );
}
