import { IServiceCollection, IServiceProvider } from 'domain/di';
import {
  addCachingOptions,
  addHttpClient,
  addHttpOptions,
  addPypiClient,
  addPypiConfig,
  addSuggestionProvider
} from './services/serviceUtils';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addPypiConfig(services);

  addHttpClient(services);

  addPypiClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("pypi", serviceProvider);
}