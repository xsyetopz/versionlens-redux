import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addDubClient,
  addDubConfig,
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

  addDubConfig(services);

  addJsonClient(services);

  addDubClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("dub", serviceProvider);
}