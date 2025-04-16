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

export function addEditorConfig(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.editorConfig,
    () => new EditorConfig(workspace)
  )
}

export function addSuggestionOptions(services: IServiceCollection) {
  services.addSingleton(
    ExtensionServiceName.suggestionOptions,
    (container: IDomainServices) => new SuggestionsOptions(container.appConfig)
  )
}

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
            const language = suggestionProvider.config.fileLanguage;
            const selectors: DocumentFilter[] = [suggestionProvider.config.filePatterns].map(
              pattern => ({
                language,
                pattern,
                scheme: 'file'
              })
            );

            // add support for JsonC
            if (language === 'json') {
              selectors.push(
                ...Array.from(
                  selectors,
                  x => ({ ...x, language: 'jsonc' })
                )
              )
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

export function addProviderNames(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.providerNames,
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
    ExtensionServiceName.editorDependencyCache,
    (container: IDomainServices) => new DependencyCache(container.providerNames)

  );
}

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