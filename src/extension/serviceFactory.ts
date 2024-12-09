import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { DependencyCache } from '#domain/packages';
import { GetSuggestions } from '#domain/useCases';
import { DisposableArray, nameOf } from '#domain/utils';
import { type IExtensionServices, VersionLensExtension } from '#extension';
import { VersionLensState } from '#extension/state';
import { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import { EditorConfig } from '#extension/vscode';
import { EventEmitter, languages, window, workspace } from 'vscode';

export function addEditorConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().editorConfig,
    () => new EditorConfig(workspace)
  )
}

export function addSuggestionOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().suggestionOptions,
    (container: IDomainServices) => new SuggestionsOptions(container.appConfig)
  )
}

export function addVersionLensState(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().versionLensState,
    async (container: IExtensionServices) => {
      const state = new VersionLensState(container.suggestionOptions)
      await state.applyDefaults();
      return state;
    }
  )
}

export function addVersionLensExtension(services: IServiceCollection) {
  const projectPath = workspace.workspaceFolders && workspace.workspaceFolders.length > 0
    ? workspace.workspaceFolders[0].uri.fsPath
    : '';

  services.addSingleton(
    nameOf<IExtensionServices>().extension,
    (container: IDomainServices & IExtensionServices) =>
      new VersionLensExtension(
        container.appConfig,
        container.versionLensState,
        container.suggestionOptions,
        projectPath
      )
  )
}

export function addOutputChannel(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().outputChannel,
    // vscode output channel called "VersionLens"
    () => window.createOutputChannel(VersionLensExtension.extensionName)
  )
}

export function addVersionLensProviders(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().versionLensProviders,
    (container: IDomainServices & IExtensionServices) =>
      new DisposableArray(
        container.suggestionProviders.map(
          suggestionProvider => {
            const provider = new SuggestionCodeLensProvider(
              container.extension,
              suggestionProvider,
              container.getSuggestions,
              new EventEmitter(),
              container.logger.child({ logGroup: `${suggestionProvider.name}CodeLensProvider` })
            );

            // register codelens provider with vscode
            const selector = suggestionProvider.config.fileMatcher;
            const expandedSelector = selector.language === 'json'
              // expand to a Array<DocumentSelector> to support JSONC
              ? [selector, { ...selector, language: 'jsonc' }]
              // otherwise use the single selector
              : selector;

            provider.disposable = languages.registerCodeLensProvider(
              expandedSelector,
              provider
            );

            return provider;
          }
        )
      ),
    true
  )
}

export function addProviderNames(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().providerNames,
    [
      'cargo',
      'composer',
      'dotnet',
      'dub',
      'maven',
      'npm',
      'pub',
      'pypi',
      'golang'
    ]
  )
}

export function addEditorDependencyCache(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IExtensionServices>().editorDependencyCache,
    (container: IDomainServices) => new DependencyCache(container.providerNames)

  );
}

export function addGetSuggestionsUseCase(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().getSuggestions;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new GetSuggestions(
        container.fetchPackages,
        [
          container.editorDependencyCache,
          container.fileWatcherDependencyCache
        ],
        container.logger.child({ logGroup: serviceName })
      )
  );
}