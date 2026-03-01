import type { ILogger } from '#domain/logging';
import type { GetSuggestionsStats } from '#domain/useCases';
import { Disposable } from '#domain/utils';
import type { VersionLensExtension } from '#extension';
import type { IVsCodeConstructFactory, IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { relative } from 'node:path';
import type { QuickPickItem } from 'vscode';

/**
 * Event handler for showing detailed suggestion statistics in a QuickPick.
 */
export class OnShowSuggestionsStatsDetails extends Disposable {

  /**
   * Initializes a new instance of the OnShowSuggestionsStatsDetails class.
   * @param getSuggestionsStats Use case for retrieving statistics.
   * @param extension The extension instance.
   * @param window VS Code window interface.
   * @param construct Factory for VS Code constructs.
   * @param logger Logger instance.
   */
  constructor(
    readonly getSuggestionsStats: GetSuggestionsStats,
    readonly extension: VersionLensExtension,
    readonly window: IVsCodeWindow,
    readonly construct: IVsCodeConstructFactory,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('getSuggestionsStats', getSuggestionsStats);
    throwUndefinedOrNull('window', window);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches stats and displays them in a grouped QuickPick list.
   * Selecting an item opens the corresponding package file.
   */
  async execute() {
    const stats = await this.getSuggestionsStats.execute(true);
    const grouped = Object.groupBy(stats, x => x.providerName);

    const items: QuickPickItem[] = []
    for (const [providerName, stats] of Object.entries(grouped)) {
      items.push({ label: providerName, kind: -1 })

      const groupPickItem = stats!.map<QuickPickItem>(
        x => ({
          label: relative(this.extension.projectPath, x.filePath),
          detail: `🟡${x.updates} 🔴${x.errors} ⚪${x.noMatches}`,
          _data: x.filePath
        })
      );

      items.push(...groupPickItem);
    }

    // show interactive choices
    const selected = await this.window.showQuickPick(
      items,
      {
        title: 'Dependency suggestions available',
        placeHolder: "Choose a file to view or press escape to cancel",
      }
    );
    if (!selected) return;

    //@ts-ignore
    await this.window.showTextDocument(this.construct.createFileUri(selected._data));
  }

}