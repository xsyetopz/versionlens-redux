import { DomainServiceName, type IDomainServices } from '#domain';
import { type IServiceCollection } from '#domain/di';
import { DependencyCache } from '#domain/packages';
import { GetSuggestions } from '#domain/useCases';
import { DisposableArray } from '#domain/utils';
import { ExtensionServiceName, VersionLensExtension, type IExtensionServices } from '#extension';
import { VersionLensState } from '#extension/state';
import { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import { EditorConfig } from '#extension/vscode';
import { EventEmitter, languages, workspace, type DocumentFilter } from 'vscode';

/**
 * Registers the VS Code editor configuration adapter as a singleton.
 * @param services The service collection to add to.
 */
export function addEditorConfig(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.editorConfig,
    () => new EditorConfig(workspace)
  )
}

/**
 * Registers the suggestion configuration options as a singleton.
 * @param services The service collection to add to.
 */
export function addSuggestionOptions(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.suggestionOptions,
    (container: IDomainServices) => new SuggestionsOptions(
      container.appConfig,
      'suggestions'
    )
  )
}

/**
 * Registers the collective extension state as a singleton and applies defaults.
 * @param services The service collection to add to.
 */
export function addVersionLensState(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.versionLensState,
    async (container: IExtensionServices) => {
      const state = new VersionLensState(container.suggestionOptions)
      await state.applyDefaults();
      return state;
    }
  )
}

/**
 * Registers the extension instance as a singleton.
 * @param services The service collection to add to.
 */
export function addVersionLensExtension(services: IServiceCollection) {
  const projectPath = workspace.workspaceFolders && workspace.workspaceFolders.length > 0
    ? workspace.workspaceFolders[0].uri.fsPath
    : '';

  services.addSingleton(
    ExtensionServiceName.extension,
    (container: IDomainServices & IExtensionServices) =>
      new VersionLensExtension(
        container.appConfig,
        container.versionLensState,
        container.suggestionOptions,
        projectPath
      )
  )
}

/**
 * Registers and initializes all ecosystem-specific CodeLens providers.
 * Each provider is registered with VS Code using document filters.
 * @param services The service collection to add to.
 */
export function addVersionLensProviders(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.versionLensProviders,
    (container: IDomainServices & IExtensionServices) =>
      new DisposableArray(
        container.suggestionProviders.map(
          suggestionProvider => {
            const provider = new SuggestionCodeLensProvider(
              container.extension,
              suggestionProvider,
              container.getSuggestions,
              new EventEmitter(),
              container.loggerFactory.create(`${suggestionProvider.name}CodeLensProvider`)
            );

            // map FileMatcher to DocumentFilter
            const fileLanguage = suggestionProvider.config.fileLanguage instanceof Array
              ? suggestionProvider.config.fileLanguage
              : [suggestionProvider.config.fileLanguage];

            const selectors: DocumentFilter[] = [];
            for (const language of fileLanguage) {
              selectors.push({
                language,
                pattern: suggestionProvider.config.filePatterns,
                scheme: 'file'
              });
            }

            // register codelens provider with vscode
            provider.disposable = languages.registerCodeLensProvider(selectors, provider);

            return provider;
          }
        )
      ),
    true
  )
}

/**
 * Registers the static list of supported provider names as a singleton.
 * @param services The service collection to add to.
 */
export function addProviderNames(services: IServiceCollection) {
  const allProviders = [
    'cargo',
    'composer',
    'docker',
    'dotnet',
    'dub',
    'maven',
    'npm',
    'deno',
    'pnpm',
    'pub',
    'pypi',
    'golang',
    'ruby'
  ];

  services.addSingleton(
    DomainServiceName.providerNames,
    () => {
      // TODO: Goal: Use a strongly typed way to get the enabledProviders setting
      const enabledProviders = workspace.getConfiguration('versionlens')
        .get<string[]>('enabledProviders') || [];

      if (enabledProviders.length > 0) {
        // Filter to ensure only supported providers are included
        return allProviders.filter(name => enabledProviders.includes(name));
      }

      return allProviders;
    }
  )
}

/**
 * Registers the cache for document-level dependencies as a singleton.
 * @param services The service collection to add to.
 */
export function addEditorDependencyCache(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.editorDependencyCache,
    (container: IDomainServices) => new DependencyCache(container.providerNames)

  );
}

/**
 * Registers the GetSuggestions use case as a singleton.
 * @param services The service collection to add to.
 */
export function addGetSuggestionsUseCase(services: IServiceCollection) {
  const serviceName = DomainServiceName.getSuggestions;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new GetSuggestions(
        container.fetchPackages,
        [
          container.editorDependencyCache,
          container.fileWatcherDependencyCache
        ],
        container.loggerFactory.create(serviceName)
      )
  );
}