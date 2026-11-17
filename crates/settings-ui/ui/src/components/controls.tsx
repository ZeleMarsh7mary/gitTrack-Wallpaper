import { useRef, useState } from "react";
import type {
  CSSProperties,
  KeyboardEvent as ReactKeyboardEvent,
  ReactNode,
} from "react";
import { Icon } from "./Icons";
import type { IconName } from "./Icons";
import { PALETTES, paletteFor, toColorInputValue } from "../lib/defaults";
import type { Position } from "../lib/types";

export function Spinner() {
  return <span className="spinner" aria-hidden="true" />;
}

export function Button({
  variant,
  onClick,
  children,
  disabled = false,
  busy = false,
  busyLabel,
}: {
  variant: "primary" | "secondary";
  onClick: () => void;
  children: ReactNode;
  disabled?: boolean;
  busy?: boolean;
  busyLabel?: string;
}) {
  return (
    <button
      type="button"
      className={`btn btn-${variant}`}
      onClick={onClick}
      disabled={disabled || busy}
      aria-busy={busy || undefined}
    >
      {busy ? <Spinner /> : null}
      <span>{busy ? (busyLabel ?? children) : children}</span>
    </button>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
}: {
  value: T;
  options: ReadonlyArray<{ value: T; label: string }>;
  onChange: (value: T) => void;
  ariaLabel: string;
}) {
  const groupRef = useRef<HTMLDivElement>(null);

  const move = (direction: number) => {
    const index = options.findIndex((option) => option.value === value);
    const next = options[(index + direction + options.length) % options.length];
    if (!next) return;
    onChange(next.value);
    requestAnimationFrame(() => {
      groupRef.current
        ?.querySelector<HTMLButtonElement>(`[data-value="${next.value}"]`)
        ?.focus();
    });
  };

  const onKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      event.preventDefault();
      move(1);
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      event.preventDefault();
      move(-1);
    }
  };

  return (
    <div
      ref={groupRef}
      className="segmented"
      role="radiogroup"
      aria-label={ariaLabel}
      onKeyDown={onKeyDown}
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="radio"
          aria-checked={option.value === value}
          tabIndex={option.value === value ? 0 : -1}
          data-value={option.value}
          className="segment"
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export function TextField({
  value,
  onChange,
  ariaLabel,
  placeholder,
  readOnly = false,
}: {
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string;
  placeholder?: string;
  readOnly?: boolean;
}) {
  return (
    <div className="field">
      <input
        type="text"
        value={value}
        placeholder={placeholder}
        aria-label={ariaLabel}
        readOnly={readOnly}
        spellCheck={false}
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </div>
  );
}

export function PasswordField({
  value,
  onChange,
  ariaLabel,
  placeholder,
}: {
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string;
  placeholder?: string;
}) {
  const [visible, setVisible] = useState(false);
  const toggleLabel = visible ? "Hide token" : "Show token";

  return (
    <div className="field field-action">
      <input
        type={visible ? "text" : "password"}
        value={value}
        placeholder={placeholder}
        aria-label={ariaLabel}
        autoComplete="off"
        spellCheck={false}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
      <button
        type="button"
        className="icon-button"
        title={toggleLabel}
        aria-label={toggleLabel}
        aria-pressed={visible}
        onClick={() => setVisible((current) => !current)}
      >
        <Icon name={visible ? "eyeOff" : "eye"} />
      </button>
    </div>
  );
}

export function Slider({
  value,
  min,
  max,
  step,
  onChange,
  format,
  ariaLabel,
}: {
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  format: (value: number) => string;
  ariaLabel: string;
}) {
  const clamped = Math.min(Math.max(value, min), max);
  const percent = max === min ? 0 : ((clamped - min) / (max - min)) * 100;
  const style = { "--fill": `${percent}%` } as unknown as CSSProperties;

  return (
    <div className="slider">
      <span className="value-badge">{format(clamped)}</span>
      <input
        type="range"
        style={style}
        min={min}
        max={max}
        step={step}
        value={clamped}
        aria-label={ariaLabel}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </div>
  );
}

export function Select({
  value,
  options,
  onChange,
  ariaLabel,
  compact = false,
}: {
  value: string;
  options: ReadonlyArray<{ value: string; label: string; disabled?: boolean }>;
  onChange: (value: string) => void;
  ariaLabel: string;
  compact?: boolean;
}) {
  return (
    <span className={`select-wrap${compact ? " is-compact" : ""}`}>
      <select
        className="select"
        value={value}
        aria-label={ariaLabel}
        onChange={(event) => onChange(event.currentTarget.value)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value} disabled={option.disabled}>
            {option.label}
          </option>
        ))}
      </select>
      <span className="select-chevron">
        <Icon name="chevron" size={12} />
      </span>
    </span>
  );
}

export function ColorSwatch({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (hex: string) => void;
}) {
  const inputValue = toColorInputValue(value, "#000000");

  return (
    <div className="color-control">
      <span className="hex">{value.toUpperCase()}</span>
      <span className="swatch" style={{ background: inputValue }} title={value.toUpperCase()}>
        <input
          type="color"
          value={inputValue}
          aria-label={label}
          onChange={(event) => onChange(event.currentTarget.value)}
        />
      </span>
    </div>
  );
}

export function PalettePicker({
  colors,
  onColorChange,
  onPreset,
}: {
  colors: readonly string[];
  onColorChange: (index: number, hex: string) => void;
  onPreset: (colors: [string, string, string, string, string]) => void;
}) {
  const preset = paletteFor(colors);

  return (
    <div className="palette-control">
      <div
        className={`palette-chips${preset ? " is-preset" : ""}`}
        role="group"
        aria-label={`Heatmap ramp${preset ? `, ${preset.name} preset` : ", custom"}`}
      >
        {colors.map((color, index) => (
          <span
            key={index}
            className="chip"
            style={{ background: color }}
            title={`Level ${index} · ${color.toUpperCase()}`}
          >
            <input
              type="color"
              value={toColorInputValue(color, "#161b22")}
              aria-label={`Level ${index} colour`}
              onChange={(event) => onColorChange(index, event.currentTarget.value)}
            />
          </span>
        ))}
      </div>
      <Select
        compact
        ariaLabel="Palette preset"
        value={preset?.name ?? ""}
        options={[
          { value: "", label: "Custom", disabled: true },
          ...PALETTES.map((item) => ({ value: item.name, label: item.name })),
        ]}
        onChange={(name) => {
          const found = PALETTES.find((item) => item.name === name);
          if (found) onPreset(found.colors);
        }}
      />
    </div>
  );
}

const ALIGN_SLOTS: ReadonlyArray<{ position: Position; label: string } | null> = [
  { position: "topLeft", label: "Top left" },
  null,
  { position: "topRight", label: "Top right" },
  null,
  { position: "center", label: "Centre" },
  null,
  { position: "bottomLeft", label: "Bottom left" },
  null,
  { position: "bottomRight", label: "Bottom right" },
];

export function AlignGrid({
  value,
  onChange,
}: {
  value: Position;
  onChange: (position: Position) => void;
}) {
  return (
    <div className="align-control">
      <div className="align-grid" role="group" aria-label="Card position">
        {ALIGN_SLOTS.map((slot, index) =>
          slot ? (
            <button
              key={slot.position}
              type="button"
              className={`align-cell${value === slot.position ? " is-active" : ""}`}
              aria-label={slot.label}
              aria-pressed={value === slot.position}
              title={slot.label}
              onClick={() => onChange(slot.position)}
            />
          ) : (
            <span key={`empty-${index}`} className="align-cell is-empty" aria-hidden="true" />
          ),
        )}
      </div>
      <button
        type="button"
        className="link-button"
        aria-pressed={value === "custom"}
        onClick={() => onChange(value === "custom" ? "center" : "custom")}
      >
        {value === "custom" ? "Snap to grid" : "Custom offsets"}
      </button>
    </div>
  );
}

export function Switch({
  checked,
  onChange,
  ariaLabel,
  disabled = false,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  ariaLabel: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      className="switch"
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span className="switch-knob" />
    </button>
  );
}

export function SettingRow({
  icon,
  title,
  subtitle,
  stacked = false,
  children,
}: {
  icon: IconName;
  title: string;
  subtitle: string;
  stacked?: boolean;
  children: ReactNode;
}) {
  return (
    <div className={`row${stacked ? " row-stacked" : ""}`}>
      <div className="row-main">
        <span className="row-icon">
          <Icon name={icon} />
        </span>
        <div className="row-text">
          <div className="row-title">{title}</div>
          <div className="row-sub">{subtitle}</div>
        </div>
      </div>
      <div className="row-control">{children}</div>
    </div>
  );
}

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="section" aria-label={title}>
      <h2 className="section-head">{title}</h2>
      <div className="rows">{children}</div>
    </section>
  );
}
