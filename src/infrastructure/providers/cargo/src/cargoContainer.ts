import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addCargoConfig,
  addCratesClient,
  addHttpOptions,
  addJsonClient,
  addSuggestionProvider
} from './services/serviceUtils';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addCargoConfig(services);

  addJsonClient(services);

  addCratesClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("cargo", serviceProvider);
}