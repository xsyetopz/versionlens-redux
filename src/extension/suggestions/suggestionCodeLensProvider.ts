import type { ILogger } from '#domain/logging';
import { type PackageResponse, defaultReplaceFn } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestions } from '#domain/useCases';
import { Disposable, nameOf } from '#domain/utils';
import { VersionLensExtension } from '#extension';
import { VersionLensState } from '#extension/state';
import { CommandFactory, SuggestionCodeLens } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';
import type {
  CancellationToken,
  CodeLens,
  CodeLensProvider,
  Event,
  EventEmitter,
  TextDocument
} from 'vscode';

const def = nameOf<SuggestionCodeLensProvider>();

export class SuggestionCodeLensProvider extends Disposable implements CodeLensProvider {

  constructor(
    readonly extension: VersionLensExtension,
    readonly suggestionProvider: ISuggestionProvider,
    readonly getSuggestions: GetSuggestions,
    readonly notifyCodeLensesChanged: EventEmitter<void>,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull(def.extension, extension);
    throwUndefinedOrNull(def.suggestionProvider, suggestionProvider);
    throwUndefinedOrNull(def.getSuggestions, getSuggestions);
    throwUndefinedOrNull(def.notifyCodeLensesChanged, notifyCodeLensesChanged);
    throwUndefinedOrNull(def.logger, logger);

    this.providerName = suggestionProvider.name;
    this.onDidChangeCodeLenses = notifyCodeLensesChanged.event;
  }

  providerName: string;

  onDidChangeCodeLenses: Event<void>;

  get state(): VersionLensState { return this.extension.state; }

  refreshCodeLenses() {
    // notify vscode to refresh version lenses
    this.notifyCodeLensesChanged.fire();
  }

  async provideCodeLenses(document: TextDocument, token: CancellationToken): Promise<CodeLens[]> {
    if (this.state.show.value === false) return [];

    const packageFilePath = document.uri.fsPath;
    const packagePath = dirname(packageFilePath);

    // get the project path from workspace path otherwise the current file
    const projectPath = this.extension.isWorkspaceMode && packagePath.startsWith(this.extension.projectPath)
      ? this.extension.projectPath
      : packagePath;

    this.logger.info("Project path is {projectPath}", projectPath);

    // clear any errors
    await this.state.clearErrorState();

    // set in progress
    await this.state.increaseBusyState();

    // fetch the package suggestions
    let suggestions: Array<PackageResponse> = [];
    try {
      suggestions = await this.getSuggestions.execute(
        this.suggestionProvider,
        projectPath,
        packageFilePath,
        this.state.showPrereleases.value
      );
    } catch (error) {
      await this.state.setErrorState();
      await this.state.clearBusyState()
      return Promise.reject(error);
    }

    await this.state.decreaseBusyState();

    // convert suggestions in to code lenses
    return SuggestionCodeLens.createFromPackageResponses(
      document,
      suggestions,
      this.suggestionProvider.suggestionReplaceFn || defaultReplaceFn
    );
  }

  async resolveCodeLens(codeLens: CodeLens, token: CancellationToken): Promise<CodeLens> {
    if (codeLens instanceof SuggestionCodeLens) {
      // evaluate the code lens
      const evaluated = this.evaluateCodeLens(codeLens);

      // enable codelens replace
      await this.state.enableCodeLensReplace(true);

      // update the progress
      return evaluated;
    }

    return codeLens;
  }

  evaluateCodeLens(codeLens: SuggestionCodeLens) {
    return CommandFactory.createSuggestedVersionCommand(
      codeLens,
      this.extension.suggestionOptions.indicators
    );
  }

  async dispose() {
    await this.disposable.dispose();
    const providerName = this.suggestionProvider.name;
    this.logger.debug(`disposed ${providerName} ${SuggestionCodeLensProvider.name}`);
  }

}