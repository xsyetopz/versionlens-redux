import type { ExtensionState } from "../state.ts";

/** The toggles of the editor win over the settings file. */
function withToggles(state: ExtensionState, value: unknown): unknown {
  const settings = JSON.parse(JSON.stringify(value ?? {})) as {
    suggestions?: Record<string, unknown>;
  };
  settings.suggestions = {
    ...settings.suggestions,
    showPrereleasesOnStartup: state.flags.showPrereleases,
  };
  return settings;
}

export { withToggles };
