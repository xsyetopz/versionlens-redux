import { DomainServiceName, IDomainServices, ServiceCollection } from '#domain';
import { CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { createJsonClient, HttpOptions, OsvClient } from '#domain/clients';
import { Config, ConfigSectionResolver } from '#domain/configuration';
import { DependencyCache, PackageCache } from '#domain/packages';
import * as Providers from '#domain/providers';
import { FileSystemStorage } from '#domain/storage';
import {
  FetchPackage,
  FetchPackages,
  GetDependencyChanges,
  GetSuggestionProvider,
  GetSuggestions,
  GetVulnerabilities,
  SortDependencies
} from '#domain/useCases';
import { ExtensionServiceName, IExtensionServices, VersionLensExtension } from '#extension';
import { VersionLensState } from '#extension/state';
import { SuggestionCodeLensProvider, SuggestionsOptions } from '#extension/suggestions';
import { EditorConfig } from '#extension/vscode';
import { VulnerabilityInteractions, VulnerabilityProvider } from '#extension/vulnerabilities';
import { EventEmitter, languages, window, workspace, type DocumentFilter } from 'vscode';
import { VsCodeConstructionFactory } from './vscode/vsCodeConstructFactory';
import { PackageFileWatcher } from './watcher';

/**
 * Registers the application configuration as a singleton.
 * @param services The service collection to add to.
 * @param configSection The name of the configuration section.
 * @param configSectionResolver The function to resolve configuration values.
 */
export function addAppConfig(
  services: ServiceCollection<IDomainServices>,
  configSection: string,
  configSectionResolver: ConfigSectionResolver
) {
  services.addSingletonFactory(
    DomainServiceName.appConfig,
    () => new Config(configSectionResolver, configSection.toLowerCase())
  )
}

export function addExtensionServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {
  services.addSingletonFactory(ExtensionServiceName.editorConfig, () => new EditorConfig(workspace))

  services.addSingletonFactory(
    ExtensionServiceName.versionLensState,
    c => new VersionLensState(c.suggestionOptions)
  )

  const projectPath = workspace.workspaceFolders && workspace.workspaceFolders.length > 0
    ? workspace.workspaceFolders[0].uri.fsPath
    : '';

  services.addSingletonFactory(
    ExtensionServiceName.extension,
    c => new VersionLensExtension(
      c.appConfig,
      c.versionLensState,
      c.suggestionOptions,
      c.vulnerabilityProvider,
      projectPath
    )
  )

}

export function addOptionServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    DomainServiceName.httpOptions,
    c => new HttpOptions(c.appConfig, 'http')
  )

  services.addSingletonFactory(
    DomainServiceName.cachingOptions,
    c => new CachingOptions(c.appConfig, 'caching')
  )

  services.addSingletonFactory(
    ExtensionServiceName.suggestionOptions,
    c => new SuggestionsOptions(c.appConfig, 'suggestions')
  )

}

export function addCachingServices(services: ServiceCollection<IDomainServices & IExtensionServices>) {

  services.addSingletonFactory(
    ExtensionServiceName.editorDependencyCache,
    c => new DependencyCache(c.providerNames)
  );

  services.addSingletonFactory(
    DomainServiceName.fileWatcherDependencyCache,
    c => new DependencyCache(c.providerNames)
  );

  services.addSingletonFactory(
    DomainServiceName.packageCache,
    c => new PackageCache(c.providerNames)
  );

  services.addSingletonValue(
    DomainServiceName.shellCache,
    new MemoryExpiryCache(DomainServiceName.shellCache)
  );

  services.addSingletonValue(
    DomainServiceName.urlRequestCache,
    new MemoryExpiryCache(DomainServiceName.urlRequestCache)
  );

  services.addSingletonValue(
    DomainServiceName.osvRequestCache,
    new MemoryExpiryCache(DomainServiceName.osvRequestCache)
  );

}

export function addStorageServices(services: ServiceCollection<IDomainServices & IExtensionServices>) {

  services.addSingletonValue(DomainServiceName.storage, new FileSystemStorage());

  services.addSingletonFactory(
    ExtensionServiceName.packageFileWatcher,
    c =>
      new PackageFileWatcher(
        c.getDependencyChanges,
        c.suggestionProviders,
        c.fileWatcherDependencyCache,
        c.editorConfig,
        workspace,
        c.loggerFactory(PackageFileWatcher)
      )
  );

}

export function addUseCaseServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {

  services.addSingletonFactory(
    DomainServiceName.getDependencyChanges,
    c => new GetDependencyChanges(
      c.storage,
      c.fileWatcherDependencyCache,
      c.loggerFactory(GetDependencyChanges)
    )
  );

  services.addSingletonFactory(
    DomainServiceName.fetchPackages,
    c => new FetchPackages(c.fetchPackage, c.loggerFactory(FetchPackages))
  );

  services.addSingletonFactory(
    DomainServiceName.fetchPackage,
    c => new FetchPackage(c.packageCache, c.loggerFactory(FetchPackage))
  );

  services.addSingletonFactory(DomainServiceName.sortDependencies, () => new SortDependencies());

}

/**
 * Registers the OsvClient as a singleton.
 * @param services The service collection to add to.
 */
export function addClientServices(services: ServiceCollection<IDomainServices>) {

  services.addSingletonFactory(
    DomainServiceName.osvClient,
    c => new OsvClient(
      c.cachingOptions,
      createJsonClient(c.authorizer, c.httpOptions),
      c.osvRequestCache
    )
  );

}

/**
 * Registers initialized suggestion providers as a singleton.
 * @param services The service collection to add to.
 */
export function addSuggestionServices(services: ServiceCollection<IExtensionServices & IDomainServices>) {
  const providerNames = Object.keys(Providers) as Array<keyof typeof Providers>;

  services.addSingletonFactory(
    DomainServiceName.GetSuggestionProvider,
    c => new GetSuggestionProvider(c.suggestionProviders)
  );

  services.addSingletonFactory(
    DomainServiceName.getSuggestions,
    c => new GetSuggestions(
      c.fetchPackages,
      [
        c.editorDependencyCache,
        c.fileWatcherDependencyCache
      ],
      c.loggerFactory(GetSuggestions)
    )
  );

  services.addSingletonFactory(
    DomainServiceName.providerNames,
    () => {
      const allProviders = Object.keys(Providers)

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

  // register the suggestionProviders array factory
  services.addSingletonFactory(
    DomainServiceName.suggestionProviders,
    c => c.providerNames
      .filter(name => providerNames.includes(name as any))
      .map(name => (c as any)[`${name}.suggestionProvider`]),
  );


  services.addSingletonFactory(
    ExtensionServiceName.suggestionCodeLensProviders,
    c => {
      const extension = c.extension;
      const getSuggestions = c.getSuggestions;
      const suggestionProviders = c.suggestionProviders;
      const loggerFactory = c.loggerFactory;
      return suggestionProviders.map(
        suggestionProvider => {
          const provider = new SuggestionCodeLensProvider(
            extension,
            suggestionProvider,
            getSuggestions,
            new EventEmitter(),
            loggerFactory(`${suggestionProvider.name}CodeLensProvider`)
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
    }
  )

  // register each provider's services
  for (const name of providerNames) {
    (Providers[name] as any).registerServices(services);
  }

}

export function addVulnerabilityServices(
  services: ServiceCollection<IExtensionServices & IDomainServices>
) {

  services.addSingletonFactory(
    DomainServiceName.getVulnerabilities,
    c => new GetVulnerabilities(c.osvClient, c.loggerFactory(GetVulnerabilities))
  );

  services.addSingletonFactory(
    ExtensionServiceName.vulnerabilityProvider,
    c => {
      const diagnostics = languages.createDiagnosticCollection('versionlens-vulnerabilities');
      const provider = new VulnerabilityProvider(
        c.versionLensState,
        c.getVulnerabilities,
        diagnostics,
        new VsCodeConstructionFactory(),
        c.suggestionOptions,
        c.loggerFactory(VulnerabilityProvider)
      );

      provider.disposables.push(diagnostics as any);

      return provider;
    }
  );

  services.addSingletonFactory(
    ExtensionServiceName.vulnerabilityInteractions,
    _ => new VulnerabilityInteractions(window)
  );

}