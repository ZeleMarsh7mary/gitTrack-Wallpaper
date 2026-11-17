import {
  PasswordField,
  Section,
  Segmented,
  SettingRow,
  Slider,
  TextField,
} from "../components/controls";
import type { UiConfig, UpdateFn } from "../lib/types";

export function SourcePage({
  config,
  update,
}: {
  config: UiConfig;
  update: UpdateFn;
}) {
  const { source } = config;
  const isRepo = source.mode === "repository";

  return (
    <Section title="GitHub connection">
      <SettingRow
        icon="branch"
        title="Source"
        subtitle="Read activity from a GitHub profile or from a single repository."
      >
        <Segmented
          ariaLabel="Git source"
          value={source.mode}
          options={[
            { value: "profile", label: "Profile" },
            { value: "repository", label: "Repository" },
          ]}
          onChange={(mode) =>
            update((current) => ({
              ...current,
              source: { ...current.source, mode },
            }))
          }
        />
      </SettingRow>

      <SettingRow
        stacked
        icon={isRepo ? "branch" : "user"}
        title={isRepo ? "Repository" : "Username"}
        subtitle={
          isRepo
            ? "Owner and repository name, for example rust-lang/rust."
            : "GitHub handle whose contributions become the wallpaper."
        }
      >
        <TextField
          value={source.inputTarget}
          placeholder={isRepo ? "rust-lang/rust" : "octocat"}
          ariaLabel={isRepo ? "Repository" : "GitHub username"}
          onChange={(inputTarget) =>
            update((current) => ({
              ...current,
              source: { ...current.source, inputTarget },
            }))
          }
        />
      </SettingRow>

      <SettingRow
        stacked
        icon="key"
        title="Personal access token"
        subtitle="Stored encrypted for this Windows account. Profile mode needs a token."
      >
        <PasswordField
          value={source.apiKey}
          ariaLabel="Personal access token"
          onChange={(apiKey) =>
            update((current) => ({
              ...current,
              source: { ...current.source, apiKey },
            }))
          }
        />
      </SettingRow>

      <SettingRow
        icon="clock"
        title="Refresh interval"
        subtitle="How often GitWallpaper re-renders the wallpaper."
      >
        <Slider
          ariaLabel="Refresh interval in minutes"
          value={source.refreshIntervalMinutes}
          min={5}
          max={240}
          step={1}
          format={(value) => `${Math.round(value)} min`}
          onChange={(refreshIntervalMinutes) =>
            update((current) => ({
              ...current,
              source: {
                ...current.source,
                refreshIntervalMinutes: Math.round(refreshIntervalMinutes),
              },
            }))
          }
        />
      </SettingRow>
    </Section>
  );
}
