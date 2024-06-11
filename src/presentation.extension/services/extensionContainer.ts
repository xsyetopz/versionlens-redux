import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  IDomainServices,
  addDomainServices,
  addFetchPackageSuggestionsUseCase,
  addFetchProjectSuggestionsUseCase
} from '#domain/services';
import { nameOf } from '#domain/utils';
import { AwilixServiceCollectionFactory } from 'infrastructure/di';
import { addInfrastructureServices } from 'infrastructure/services';
import {
  VersionLensExtension,
  addEditorDependencyCache,
  addGetSuggestionsUseCase,
  addOnActiveTextEditorChange,
  addOnClearCache,
  addOnErrorClick,
  addOnFileLinkClick,
  addOnPackageDependenciesChanged,
  addOnPreSaveChanges,
  addOnProviderEditorActivated,
  addOnProviderTextDocumentChange,
  addOnProviderTextDocumentClose,
  addOnSaveChanges,
  addOnTextDocumentChange,
  addOnTextDocumentClose,
  addOnTextDocumentSave,
  addOnTogglePrereleases,
  addOnToggleReleases,
  addOnUpdateDependencyClick,
  addOutputChannel,
  addProviderNames,
  addSuggestionOptions,
  addVersionLensExtension,
  addVersionLensProviders,
  addVersionLensState
} from 'presentation.extension';
import { ExtensionContext, workspace } from 'vscode';

export async function configureContainer(context: ExtensionContext): Promise<IServiceProvider> {
  const serviceCollectionFactory = new AwilixServiceCollectionFactory();
  const services = serviceCollectionFactory.createServiceCollection();
  services.addSingleton(
    nameOf<IDomainServices>().serviceCollectionFactory,
    serviceCollectionFactory
  );

  // domain
  addDomainServices(
    services,
    VersionLensExtension.extensionName,
    workspace.getConfiguration
  );

  // infrastructure
  const defaultLogGroup = "extension";
  addInfrastructureServices(services, defaultLogGroup);

  // extension
  addExtensionServices(services)

  return await services.build();
}

function addExtensionServices(services: IServiceCollection) {
  addProviderNames(services);
  addSuggestionOptions(services);
  addVersionLensState(services);
  addVersionLensExtension(services);
  addOutputChannel(services);
  addVersionLensProviders(services);
  addEditorDependencyCache(services);
  addFetchProjectSuggestionsUseCase(services);
  addFetchPackageSuggestionsUseCase(services);
  addGetSuggestionsUseCase(services);

  // events
  addEventServices(services);
}

function addEventServices(services) {
  // commands
  addOnClearCache(services);
  addOnFileLinkClick(services);
  addOnUpdateDependencyClick(services);

  // editorTitleBar
  addOnErrorClick(services);
  addOnToggleReleases(services);
  addOnTogglePrereleases(services);

  // install
  addOnPreSaveChanges(services);
  addOnSaveChanges(services);

  // provider
  addOnProviderEditorActivated(services);
  addOnProviderTextDocumentChange(services);
  addOnProviderTextDocumentClose(services);

  // vscode
  addOnActiveTextEditorChange(services);
  addOnTextDocumentChange(services);
  addOnTextDocumentClose(services);
  addOnTextDocumentSave(services);

  // watcher
  addOnPackageDependenciesChanged(services);
}