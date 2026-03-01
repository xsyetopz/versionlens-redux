import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addDenoConfig,
  addDenoSuggestionResolver,
  addHttpOptions,
  addJsrClient,
  addSuggestionProvider
} from '#domain/providers/deno';

/**
 * Configures the Deno service container by registering all necessary services.
 * @param serviceProvider The root service provider.
 * @param services The service collection to configure.
 * @returns A promise that resolves to the newly built child service provider.
 */
export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addDenoConfig(services);

  addJsrClient(services);

  addDenoSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("deno", serviceProvider);
}