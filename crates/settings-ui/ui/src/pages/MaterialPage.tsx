import {
  AlignGrid,
  ColorSwatch,
  Section,
  SettingRow,
  Select,
  Slider,
} from "../components/controls";
import type { UiConfig, UpdateFn } from "../lib/types";

export function MaterialPage({
  config,
  update,
}: {
  config: UiConfig;
  update: UpdateFn;
}) {
  const { card } = config;
  const hasSurface = card.material !== "none";
  const isFrosted =
    card.material === "glass" ||
    card.material === "acrylic" ||
    card.material === "mica";

  return (
    <Section title="Card">
      <SettingRow
        icon="move"
        title="Position"
        subtitle="Anchor the contribution card to a corner or the centre."
      >
        <AlignGrid
          value={card.position}
          onChange={(position) =>
            update((current) => ({
              ...current,
              card: {
                ...current.card,
                position,
                // Preset slots are absolute; keeping stale custom offsets
                // would shift the card with no visible control.
                offsetX: position === "custom" ? current.card.offsetX : 0,
                offsetY: position === "custom" ? current.card.offsetY : 0,
              },
            }))
          }
        />
      </SettingRow>

      {card.position === "custom" ? (
        <>
          <SettingRow
            icon="arrowX"
            title="Horizontal offset"
            subtitle="Shift the card from the centre of the screen."
          >
            <Slider
              ariaLabel="Horizontal offset"
              value={card.offsetX}
              min={-1200}
              max={1200}
              step={1}
              format={(value) =>
                `${Math.round(value) > 0 ? "+" : ""}${Math.round(value)} px`
              }
              onChange={(offsetX) =>
                update((current) => ({
                  ...current,
                  card: { ...current.card, offsetX: Math.round(offsetX) },
                }))
              }
            />
          </SettingRow>

          <SettingRow
            icon="arrowY"
            title="Vertical offset"
            subtitle="Shift the card up or down from the centre."
          >
            <Slider
              ariaLabel="Vertical offset"
              value={card.offsetY}
              min={-800}
              max={800}
              step={1}
              format={(value) =>
                `${Math.round(value) > 0 ? "+" : ""}${Math.round(value)} px`
              }
              onChange={(offsetY) =>
                update((current) => ({
                  ...current,
                  card: { ...current.card, offsetY: Math.round(offsetY) },
                }))
              }
            />
          </SettingRow>
        </>
      ) : null}

      <SettingRow
        icon="layers"
        title="Material"
        subtitle="Backdrop treatment behind the heatmap."
      >
        <Select
          ariaLabel="Card material"
          value={card.material}
          options={[
            { value: "none", label: "None" },
            { value: "solid", label: "Solid" },
            { value: "glass", label: "Glass" },
            { value: "acrylic", label: "Acrylic" },
            { value: "mica", label: "Mica" },
          ]}
          onChange={(material) =>
            update((current) => ({
              ...current,
              card: {
                ...current.card,
                material: material as UiConfig["card"]["material"],
              },
            }))
          }
        />
      </SettingRow>

      {hasSurface ? (
        <SettingRow
          icon="droplet"
          title="Tint colour"
          subtitle="Base colour the material is mixed from."
        >
          <ColorSwatch
            label="Card tint colour"
            value={card.tintColor}
            onChange={(tintColor) =>
              update((current) => ({
                ...current,
                card: { ...current.card, tintColor },
              }))
            }
          />
        </SettingRow>
      ) : null}

      {hasSurface ? (
        <SettingRow
          icon="circleHalf"
          title="Opacity"
          subtitle="How much of the wallpaper shows through the card."
        >
          <Slider
            ariaLabel="Card opacity"
            value={card.opacity}
            min={0}
            max={1}
            step={0.01}
            format={(value) => value.toFixed(2)}
            onChange={(opacity) =>
              update((current) => ({
                ...current,
                card: {
                  ...current.card,
                  opacity: Math.round(opacity * 100) / 100,
                },
              }))
            }
          />
        </SettingRow>
      ) : null}

      {isFrosted ? (
        <SettingRow
          icon="blur"
          title="Blur radius"
          subtitle="Soften the wallpaper behind the card."
        >
          <Slider
            ariaLabel="Blur radius"
            value={card.blurRadius}
            min={0}
            max={50}
            step={1}
            format={(value) => `${Math.round(value)} px`}
            onChange={(blurRadius) =>
              update((current) => ({
                ...current,
                card: { ...current.card, blurRadius: Math.round(blurRadius) },
              }))
            }
          />
        </SettingRow>
      ) : null}

      <SettingRow
        icon="corner"
        title="Corner radius"
        subtitle="Round the corners of the card."
      >
        <Slider
          ariaLabel="Corner radius"
          value={card.cornerRadius}
          min={0}
          max={80}
          step={1}
          format={(value) => `${Math.round(value)} px`}
          onChange={(cornerRadius) =>
            update((current) => ({
              ...current,
              card: { ...current.card, cornerRadius: Math.round(cornerRadius) },
            }))
          }
        />
      </SettingRow>
    </Section>
  );
}
