import { IServiceCollection } from "domain/di";
import { DependencyCache } from "domain/packages";
import { IDomainServices } from "domain/services";
import { GetSuggestions } from "domain/useCases";
import { DisposableArray, nameOf } from 'domain/utils';
import {
  IExtensionServices,
  OnActiveTextEditorChange,
  OnClearCache,
  OnErrorClick,
  OnFileLinkClick,
  OnPackageDependenciesChanged,
  OnPreSaveChanges,
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose,
  OnSaveChanges,
  OnTextDocumentChange,
  OnTextDocumentClose,
  OnTextDocumentSave,
  OnTogglePrereleases,
  OnToggleReleases,
  OnUpdateDependencyClick,
  SuggestionCodeLensProvider,
  SuggestionsOptions,
  VersionLensExtension,
  VersionLensState
} from "presentation.extension";
import { window, workspace } from "vscode";

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

export function addOnActiveTextEditorChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onActiveTextEditorChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnActiveTextEditorChange(
        container.extension.state,
        container.GetSuggestionProvider,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnTextDocumentChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnTextDocumentChange(
        container.GetSuggestionProvider,
        container.versionLensState,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnTextDocumentClosed(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentClose;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnTextDocumentClose(
        container.GetSuggestionProvider,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnTextDocumentSave(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentSave;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnTextDocumentSave(
        container.GetSuggestionProvider,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnProviderEditorActivated(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderEditorActivated;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const listener = new OnProviderEditorActivated(
        container.loggerChannel,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onActiveTextEditorChange.registerListener(listener.execute, listener);
      return listener;
    },
    false
  )
}

export function addOnProviderTextDocumentChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const listener = new OnProviderTextDocumentChange(
        container.extension.state,
        container.getDependencyChanges,
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentChange.registerListener(listener.execute, listener);
      return listener;
    },
    false
  )
}

export function addOnClearCache(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onClearCache;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnClearCache(
        container.packageCache,
        container.processesCache,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnPreSaveChanges(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onPreSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const event = new OnPreSaveChanges(
        container.fileWatcherDependencyCache,
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentSave.registerListener(event.execute, event, 1);

      return event;
    }
  )
}

export function addOnSaveChanges(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const event = new OnSaveChanges(
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentSave.registerListener(event.execute, event, 2);

      return event;
    }
  )
}

export function addOnFileLinkClick(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onFileLinkClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => {
      return new OnFileLinkClick(
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnUpdateDependencyClick(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onUpdateDependencyClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => {
      return new OnUpdateDependencyClick(
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnToggleReleases(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onToggleReleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnToggleReleases(
        container.versionLensProviders,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnTogglePrereleases(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTogglePrereleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnTogglePrereleases(
        container.versionLensProviders,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnShowError(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onErrorClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnErrorClick(
        container.extension.state,
        container.outputChannel,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnPackageDependenciesChanged(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onPackageDependenciesChanged
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const event = new OnPackageDependenciesChanged(
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.packageFileWatcher.registerListener(event.execute, event);

      return event;
    }
  )
}

export function addOnProviderTextDocumentClose(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderTextDocumentClose
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const event = new OnProviderTextDocumentClose(
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentClose.registerListener(event.execute, event);

      return event;
    }
  )
}