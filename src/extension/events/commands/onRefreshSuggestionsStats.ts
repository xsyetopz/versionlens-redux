import type { ILogger } from '#domain/logging';
import type { GetSuggestionsStats } from '#domain/useCases';
import { Disposable } from '#domain/utils';
import type { IVersionLensState } from '#extension';
import type { SuggestionsOptions } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { StatusBarItem } from 'vscode';

/**
 * Event handler for refreshing suggestion statistics shown in the status bar.
 */
export class OnRefreshSuggestionsStats extends Disposable {

  /**
   * Initializes a new instance of the OnRefreshSuggestionsStats class.
   * @param statusBarItem The VS Code status bar item to update.
   * @param getSuggestionsStats Use case for calculating statistics.
   * @param state Extension state.
   * @param options Suggestion configuration options.
   * @param logger Logger instance.
   */
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

  /**
   * Refreshes the statistics and updates the status bar item.
   * @param useCache Whether to use cached stats if available.
   */
  async execute(useCache: boolean) {
    if (this.state.showSuggestionsStats.value === false) {
      this.statusBarItem.hide();
      return;
    }

    this.statusBarItem.text = 'V $(loading~spin)';
    this.statusBarItem.show();

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