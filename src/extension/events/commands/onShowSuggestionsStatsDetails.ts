import type { ILogger } from '#domain/logging';
import type { GetSuggestionsStats } from '#domain/useCases';
import { Disposable } from '#domain/utils';
import type { VersionLensExtension } from '#extension';
import type { IVsCodeConstructFactory, IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { relative } from 'node:path';
import type { QuickPickItem } from 'vscode';

export class OnShowSuggestionsStatsDetails extends Disposable {

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