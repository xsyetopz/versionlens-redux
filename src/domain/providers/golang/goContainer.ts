import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addGoClient,
  addGoConfig,
  addHttpClient,
  addHttpOptions,
  addSuggestionProvider
} from '#domain/providers/golang';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addGoConfig(services);

  addHttpClient(services);

  addGoClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("golang", serviceProvider);
}