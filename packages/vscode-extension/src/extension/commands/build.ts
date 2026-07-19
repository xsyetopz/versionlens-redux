import { window } from "#vscode-host";
import type { ExtensionState } from "../state.ts";
import { applyRustEdits } from "./apply.ts";

interface BuildOptions {
  builds: string[];
  currentVersion: string | undefined;
  packageName: string | undefined;
}

export async function chooseBuild(
  state: ExtensionState,
  selector: string | undefined,
  options: BuildOptions,
): Promise<void> {
  const { builds, currentVersion, packageName } = options;
  if (
    !selector ||
    builds.length === 0 ||
    state.flags.codeLensReplace === false
  ) {
    return;
  }

  const selected = await window.showQuickPick(
    builds.map((build): { label: string; picked: boolean } => ({
      label: build,
      picked: build === currentVersion,
    })),
    {
      placeHolder: "Choose a build or press escape to cancel",
      title: `Choose a build for ${packageName ?? ""}`.trim(),
    },
  );
  if (!selected) {
    return;
  }

  await applyRustEdits(state, selector, undefined, {
    selectedVersion: selected.label,
  });
}
