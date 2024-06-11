import { IServiceCollection } from '#domain/di';
import { DependencyCache } from '#domain/packages';
import { IDomainServices } from 'domain/services';
import { GetSuggestions } from 'domain/useCases';
import { DisposableArray, nameOf } from 'domain/utils';
import {
  IExtensionServices,
  SuggestionCodeLensProvider,
  SuggestionsOptions,
  VersionLensExtension,
  VersionLensState
} from 'presentation.extension';
import { window, workspace } from 'vscode';

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
          suggestionProvider => new SuggestionCodeLensProvider(
            container.extension,
            suggestionProvider,
            container.getSuggestions,
            container.logger.child({ logGroup: `${suggestionProvider.name}CodeLensProvider` })
          )
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
        container.fetchProjectSuggestions,
        [
          container.editorDependencyCache,
          container.fileWatcherDependencyCache
        ],
        container.logger.child({ logGroup: serviceName })
      )
  );
}