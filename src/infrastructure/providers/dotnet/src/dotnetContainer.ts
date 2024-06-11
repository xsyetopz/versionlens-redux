import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addCliClient,
  addDotNetConfig,
  addHttpOptions,
  addJsonClient,
  addNugetOptions,
  addNuGetPackageClient,
  addNuGetResourceClient,
  addProcessClient,
  addSuggestionProvider
} from './services/serviceUtils';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addNugetOptions(services);

  addHttpOptions(services);

  addDotNetConfig(services);

  addProcessClient(services);

  addCliClient(services);

  addJsonClient(services);

  addNuGetPackageClient(services);

  addNuGetResourceClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("dotnet", serviceProvider);
}