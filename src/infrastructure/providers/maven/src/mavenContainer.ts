import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addCliClient,
  addHttpClient,
  addHttpOptions,
  addMavenClient,
  addMavenConfig,
  addProcessClient,
  addSuggestionProvider
} from './services/serviceUtils';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addMavenConfig(services);

  addProcessClient(services);

  addCliClient(services);

  addHttpClient(services);

  addMavenClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("maven", serviceProvider);
}