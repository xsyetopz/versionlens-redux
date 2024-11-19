import type { ILogger } from '#domain/logging';
import { type PackageResponse, defaultReplaceFn } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestions } from '#domain/useCases';
import type { IDisposable } from '#domain/utils';
import { VersionLensExtension } from '#extension';
import { VersionLensState } from '#extension/state';
import { CommandFactory, SuggestionCodeLens } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';
import * as VsCode from 'vscode';
import {
  type CancellationToken,
  type CodeLensProvider,
  type Event,
  type TextDocument,
  CodeLens,
  EventEmitter,
  languages
} from 'vscode';

export class SuggestionCodeLensProvider implements CodeLensProvider, IDisposable {

  constructor(
    readonly extension: VersionLensExtension,
    readonly suggestionProvider: ISuggestionProvider,
    readonly getSuggestions: GetSuggestions,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("extension", extension);
    throwUndefinedOrNull("suggestionProvider", suggestionProvider);
    throwUndefinedOrNull("getSuggestions", getSuggestions);
    throwUndefinedOrNull("logger", logger);

    this.providerName = suggestionProvider.name;

    // register changed event before registering the codelens
    this.notifyCodeLensesChanged = new EventEmitter();
    this.onDidChangeCodeLenses = this.notifyCodeLensesChanged.event;

    // register the codelens provider with vscode
    const selector = suggestionProvider.config.fileMatcher;

    // check if the provider has a JSON selector
    const expandedSelector = selector.language === 'json'
      // expand to a Array<DocumentSelector> to support JSONC
      ? [selector, { ...selector, language: 'jsonc' }]
      // otherwise use the single selector
      : selector;

    // register provider
    this.disposable = languages.registerCodeLensProvider(
      expandedSelector,
      this
    );
  }

  providerName: string;

  notifyCodeLensesChanged: EventEmitter<void>;

  onDidChangeCodeLenses: Event<void>;

  disposable: VsCode.Disposable

  get state(): VersionLensState {
    return this.extension.state;
  }

  reloadCodeLenses() {
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

    this.logger.info("Project path is %s", projectPath);

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