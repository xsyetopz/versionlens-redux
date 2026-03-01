import type { IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { QuickPickItem } from 'vscode';

/**
 * Handles user interactions specifically for package version suggestions.
 */
export class SuggestionInteractions {

  /**
   * Initializes a new instance of the SuggestionInteractions class.
   * @param window The VS Code window interface.
   */
  constructor(readonly window: IVsCodeWindow) {
    throwUndefinedOrNull('window', window);
  }

  /**
   * Prompts the user to choose a specific build from a list.
   * @param buildVersions The list of available build versions.
   * @param packageName The name of the package.
   * @param packageVersion The current version of the package.
   * @returns A promise resolving to the selected build string, or undefined if cancelled.
   */
  async chooseBuild(buildVersions: string[], packageName: string, packageVersion: string): Promise<string | undefined> {
    const pickItems = buildVersions.map(x => <QuickPickItem>{ label: x, picked: x === packageVersion });
    // show interactive choices
    const selected = await this.window.showQuickPick(
      pickItems,
      {
        title: `Choose a build for ${packageName}`,
        placeHolder: "Choose a build or press escape to cancel",
      }
    );
    if (!selected) return;

    return selected.label;
  }


}