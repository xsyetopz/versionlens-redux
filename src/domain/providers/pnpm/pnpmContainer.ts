import type { IServiceCollection, IServiceProvider } from '#domain/di';
import { addPnpmConfig, addSuggestionProvider } from './serviceFactory.js';

/**
 * Configures the PNPM service container by registering all necessary services.
 * @param serviceProvider The root service provider.
 * @param services The service collection to configure.
 * @returns A promise that resolves to the newly built child service provider.
 */
export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addPnpmConfig(services);

  addSuggestionProvider(services);

  return await services.buildChild('pnpm', serviceProvider);
}