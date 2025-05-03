import type { ILogger } from '#domain/logging';
import type { GetSuggestionsStats } from '#domain/useCases';
import { Disposable } from '#domain/utils';
import type { SuggestionsOptions } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { IVersionLensState } from 'src/extension/definitions.js';
import type { StatusBarItem } from 'vscode';

export class OnRefreshSuggestionsStats extends Disposable {

  constructor(
    readonly statusBarItem: StatusBarItem,
    readonly getSuggestionsStats: GetSuggestionsStats,
    readonly state: IVersionLensState,
    readonly options: SuggestionsOptions,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('statusBarItem', statusBarItem);
    throwUndefinedOrNull('getSuggestionsStats', getSuggestionsStats);
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('options', options);
    throwUndefinedOrNull('logger', logger);
  }

  async execute(useCache: boolean) {
    if (this.state.showSuggestionsStats.value === false) {
      this.statusBarItem.hide();
      return;
    }

    const handle = startStatusBarProgress(this.statusBarItem);

    this.logger.info("fetching all suggestion stats");
    // capture start time
    const startedAt = performance.now();
    const stats = await this.getSuggestionsStats.execute(useCache);

    let noMatches = 0;
    let updates = 0;
    let errors = 0;
    for (const stat of stats) {
      noMatches += stat.noMatches;
      updates += stat.updates;
      errors += stat.errors;
    }

    // stop progress
    clearInterval(handle)

    // update status item text
    this.statusBarItem.text = `V ${updates + errors + noMatches}`;
    const tooltipBuilder: string[] = [
      `${this.options.indicators.Match}${updates}`,
      `${this.options.indicators.Error}${errors}`,
      `${this.options.indicators.NoMatch}${noMatches}`
    ];
    this.statusBarItem.tooltip = tooltipBuilder.join(' ');

    // report completed duration
    const completedAt = performance.now();
    this.logger.info(
      "completed fetching all suggestion stats ({duration} ms)",
      Math.floor(completedAt - startedAt)
    );
  }
}

const progressChars = ['◷', '◶', '◵', '◴'];
function startStatusBarProgress(statusBarItem: StatusBarItem) {
  let pos = 0;
  statusBarItem.text = `V ${progressChars[pos]}`
  statusBarItem.show();
  return setInterval(
    () => {
      statusBarItem.text = `V ${progressChars[pos]}`
      pos = (pos + 1) % progressChars.length
    },
    200
  );
}