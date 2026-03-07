import type { ILogger } from '#domain/logging';
import { type PackageResponse, defaultReplaceFn } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestions } from '#domain/useCases';
import { Disposable, nameOf } from '#domain/utils';
import { VersionLensExtension } from '#extension';
import { VersionLensState } from '#extension/state';
import { CommandFactory, createFromPackageResponses, SuggestionCodeLens } from '#extension/suggestions';
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

/**
 * VS Code CodeLens provider for package version suggestions.
 */
export class SuggestionCodeLensProvider extends Disposable implements CodeLensProvider {

  /**
   * Initializes a new instance of the SuggestionCodeLensProvider class.
   * @param extension The extension instance.
   * @param suggestionProvider The underlying suggestion provider for a specific ecosystem.
   * @param getSuggestions Use case for retrieving version suggestions.
   * @param notifyCodeLensesChanged EventEmitter to notify VS Code when lenses need refreshing.
   * @param logger Logger instance.
   */
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

  /** The name of the suggestion provider. */
  providerName: string;

  /** Event that fires when code lenses change. */
  onDidChangeCodeLenses: Event<void>;

  /** Gets the extension state. */
  get state(): VersionLensState { return this.extension.state; }

  /**
   * Triggers a refresh of all version lenses.
   */
  refreshCodeLenses() {
    // notify vscode to refresh version lenses
    this.notifyCodeLensesChanged.fire();
  }

  /**
   * Provides the initial list of code lenses for a document.
   * Fetches suggestions and converts them to lenses.
   * @param document The VS Code text document.
   * @param token A cancellation token.
   * @returns A promise resolving to an array of code lenses.
   */
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
      await this.state.clearBusyState();
      return Promise.reject(error);
    } finally {
      await this.state.decreaseBusyState();
      this.logger.debug(
        "finished fetching suggestions for {providerName}",
        this.providerName
      );
    }

    // convert suggestions in to code lenses
    return createFromPackageResponses(
      document,
      suggestions,
      this.suggestionProvider.suggestionReplaceFn || defaultReplaceFn
    );
  }

  /**
   * Resolves the command for a code lens.
   * Evaluates the suggestion and assigns the appropriate VS Code command.
   * @param codeLens The code lens to resolve.
   * @param token A cancellation token.
   * @returns A promise resolving to the fully resolved code lens.
   */
  async resolveCodeLens(codeLens: CodeLens, token: CancellationToken): Promise<CodeLens | undefined> {
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

  /**
   * Internal method to evaluate a code lens and assign its command.
   */
  evaluateCodeLens(codeLens: SuggestionCodeLens) {
    return CommandFactory.createSuggestedVersionCommand(
      codeLens,
      this.extension.suggestionOptions.indicators
    );
  }

  /**
   * Disposes of the provider resources.
   */
  async dispose() {
    await this.disposable.dispose();
    const providerName = this.suggestionProvider.name;
    this.logger.debug(`disposed ${providerName} ${SuggestionCodeLensProvider.name}`);
  }

}