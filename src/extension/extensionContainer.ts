import { addDomainServices, DomainServiceName } from '#domain';
import type { IServiceCollection, IServiceProvider } from '#domain/di';
import { AwilixServiceCollectionFactory } from '#domain/di/awilix';
import {
  type ExtensionContext,
  type Memento,
  type SecretStorage,
  workspace
} from 'vscode';
import {
  addAuthenticationInteractions,
  addAuthenticationProviders,
  addAuthorizer,
  addUrlAuthenticationStore
} from './authorization/serviceFactory';
import {
  addOnActiveTextEditorChange,
  addOnAddUrlAuthentication,
  addOnChooseBuildClick,
  addOnClearCache,
  addOnErrorClick,
  addOnFileLinkClick,
  addOnPackageDependenciesChanged,
  addOnProviderEditorActivated,
  addOnProviderTextDocumentChange,
  addOnProviderTextDocumentClose,
  addOnRefreshSuggestionsStats,
  addOnRemoveUrlAuthentication,
  addOnSaveChanges,
  addOnShowSuggestionsStatsDetails,
  addOnTextDocumentChange,
  addOnTextDocumentClose,
  addOnTextDocumentSave,
  addOnTogglePrereleases,
  addOnToggleReleases,
  addOnUpdateDependencyClick
} from './events/serviceFactory';
import { addLoggerSinks, addLogOutputChannel } from './logging/serviceFactory';
import {
  addEditorConfig,
  addEditorDependencyCache,
  addGetSuggestionsUseCase,
  addProviderNames,
  addSuggestionOptions,
  addVersionLensExtension,
  addVersionLensProviders,
  addVersionLensState
} from './serviceFactory';
import { VersionLensExtension } from './versionLensExtension';
import { addPackageFileWatcher } from './watcher/serviceFactory';

export async function configureContainer(
  context: ExtensionContext,
  resourceFolderPath: string
): Promise<IServiceProvider> {
  const serviceCollectionFactory = new AwilixServiceCollectionFactory();
  const services = serviceCollectionFactory.createServiceCollection();
  services.addSingleton(
    DomainServiceName.serviceCollectionFactory,
    serviceCollectionFactory
  );

  // domain
  addDomainServices(
    services,
    VersionLensExtension.extensionName,
    workspace.getConfiguration
  );

  // extension
  addExtensionServices(
    services,
    resourceFolderPath,
    context.workspaceState,
    context.secrets
  );

  return await services.build();
}

function addExtensionServices(
  services: IServiceCollection,
  resourceFolderPath: string,
  workspaceState: Memento,
  secrets: SecretStorage
) {
  addEditorConfig(services);
  addProviderNames(services);
  addSuggestionOptions(services);
  addVersionLensState(services);
  addVersionLensExtension(services);
  addVersionLensProviders(services);
  addEditorDependencyCache(services);
  addGetSuggestionsUseCase(services);

  // logging
  addLogOutputChannel(services);
  addLoggerSinks(services);

  // file watcher
  addPackageFileWatcher(services);

  // auth
  addAuthenticationProviders(services, resourceFolderPath, secrets);
  addAuthenticationInteractions(services);
  addUrlAuthenticationStore(services, workspaceState);
  addAuthorizer(services);

  // auth events
  addOnAddUrlAuthentication(services);
  addOnRemoveUrlAuthentication(services);

  // command events
  addOnClearCache(services);
  addOnFileLinkClick(services);
  addOnUpdateDependencyClick(services);
  addOnChooseBuildClick(services);
  addOnRefreshSuggestionsStats(services);
  addOnShowSuggestionsStatsDetails(services);

  // editorTitleBar events
  addOnErrorClick(services);
  addOnToggleReleases(services);
  addOnTogglePrereleases(services);

  // install event
  addOnSaveChanges(services);

  // provider events
  addOnProviderEditorActivated(services);
  addOnProviderTextDocumentChange(services);
  addOnProviderTextDocumentClose(services);

  // vscode events
  addOnActiveTextEditorChange(services);
  addOnTextDocumentChange(services);
  addOnTextDocumentClose(services);
  addOnTextDocumentSave(services);

  // watcher events
  addOnPackageDependenciesChanged(services);
}