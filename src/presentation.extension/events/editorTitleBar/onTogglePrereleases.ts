import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import { Disposable } from 'domain/utils';
import {
  IconCommandFeatures,
  SuggestionCodeLensProvider,
  VersionLensState
} from 'presentation.extension';
import { commands } from 'vscode';

export class OnTogglePrereleases extends Disposable {

  constructor(
    readonly suggestionCodeLensProviders: SuggestionCodeLensProvider[],
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("suggestionCodeLensProviders", suggestionCodeLensProviders);
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("logger", logger);

    // register the vscode commands
    this.disposables.push(
      commands.registerCommand(
        IconCommandFeatures.ShowPrereleaseVersions,
        this.execute.bind(this, true)
      ),
      commands.registerCommand(
        IconCommandFeatures.HidePrereleaseVersions,
        this.execute.bind(this, false)
      )
    );
  }

  async execute(toggle: boolean): Promise<void> {
    await this.state.showPrereleases.change(toggle);

    // refresh the active code lenses
    const providerName = this.state.providerActive.value;
    const codelensProvider = this.suggestionCodeLensProviders.find(
      x => x.providerName === providerName
    );

    codelensProvider && codelensProvider.reloadCodeLenses();
  }

}