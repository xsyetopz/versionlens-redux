import {
  DomainServiceName,
  type IDomainServices,
  ServiceCollection,
  type ServiceProvider
} from '#domain';
import { type ExtensionContext, workspace } from 'vscode';
import { addAuthenticationServices } from './authorization/serviceFactory';
import type { IExtensionServices } from './definitions';
import { addEventServices } from './events/serviceFactory';
import { addLoggingServices } from './logging/serviceFactory';
import {
  addAppConfig,
  addCachingServices,
  addClientServices,
  addExtensionServices,
  addOptionServices,
  addStorageServices,
  addSuggestionServices,
  addUseCaseServices,
  addVulnerabilityServices
} from './serviceFactory';
import { VersionLensExtension } from './versionLensExtension';

/**
 * Configures the dependency injection container for the entire extension.
 * This includes domain services and extension-specific services.
 * @param context The VS Code extension context.
 * @param resourceFolderPath The path to the resource folder for the current context.
 * @returns A promise resolving to the initialized service provider.
 */
export function configureContainer(
  context: ExtensionContext,
  resourceFolderPath: string
): ServiceProvider<IDomainServices & IExtensionServices> {
  const services = new ServiceCollection<IDomainServices & IExtensionServices>();

  addAppConfig(services, VersionLensExtension.extensionName, workspace.getConfiguration);
  addOptionServices(services);
  addStorageServices(services);
  addClientServices(services);
  addUseCaseServices(services);
  addExtensionServices(services);
  addLoggingServices(services);
  addCachingServices(services);
  addAuthenticationServices(services, resourceFolderPath, context.secrets, context.workspaceState);
  addEventServices(services);
  addSuggestionServices(services);
  addVulnerabilityServices(services);

  // register the service provider itself
  services.addServiceProvider(DomainServiceName.serviceProvider);

  return services.build();
}