import { DomainServiceName } from '#domain';
import type { IServiceCollectionFactory, IServiceProvider } from '#domain/di';
import type { ILogger } from '#domain/logging';
import type {
  IProviderModule,
  IProviderServices,
  ISuggestionProvider
} from '#domain/providers';
import { nameOf } from '#domain/utils';

/**
 * Imports multiple suggestion providers by name.
 * @param serviceProvider The root service provider.
 * @param providerNames An array of provider names to import.
 * @param logger The logger to use.
 * @returns A promise resolving to an array of imported suggestion providers.
 */
export function importSuggestionProviders(
  serviceProvider: IServiceProvider,
  providerNames: Array<string>,
  logger: ILogger
): Promise<Array<ISuggestionProvider>> {

  logger.debug('Loading suggestion providers {providerNames}', providerNames);

  const promises = [];

  for (const providerName of providerNames) {
    const promise = importSuggestionProvider(
      serviceProvider,
      providerName,
      logger
    );
    promises.push(promise);
  }

  // parallel the promises
  return Promise.all(promises);
}

/**
 * Imports a single suggestion provider by name and configures its container.
 * @param serviceProvider The root service provider.
 * @param providerName The name of the provider to import.
 * @param logger The logger to use.
 * @returns A promise resolving to the imported suggestion provider.
 */
export async function importSuggestionProvider(
  serviceProvider: IServiceProvider,
  providerName: string,
  logger: ILogger
): Promise<ISuggestionProvider> {

  try {

    logger.debug('Activating container scope for {providerName}', providerName);

    // get the service collection factory
    const serviceCollectionFactory = serviceProvider.getService<IServiceCollectionFactory>(
      DomainServiceName.serviceCollectionFactory
    );

    // import the provider
    let module: IProviderModule;
    // works around an esbuild bug with string literal templates and tsconfig.json paths
    // https://github.com/evanw/esbuild/issues/3798
    try {
      // bundle mode
      module = await import(`../../domain/providers/${providerName}/index.ts`);
    } catch (e) {
      // dev mode
      module = await import(`#domain/providers/${providerName}`)
    }

    // register the provider
    const childServiceProvider = await module.configureContainer(
      serviceProvider,
      serviceCollectionFactory.createServiceCollection()
    );

    const suggestionProvider = childServiceProvider.getService<ISuggestionProvider>(
      nameOf<IProviderServices>().suggestionProvider
    );

    serviceProvider.registerService(childServiceProvider.name, childServiceProvider);

    logger.debug(
      "Registered provider for {providerName}:\t file pattern: {filePatterns}\t caching: {cacheDuration} seconds\t strict ssl: {strictSSL}",
      providerName,
      suggestionProvider.config.filePatterns,
      suggestionProvider.config.caching.duration / 1000,
      suggestionProvider.config.http.strictSSL,
    );

    return suggestionProvider;

  } catch (error) {

    logger.error(
      'Could not register provider {providerName}. Reason: {error}',
      providerName,
      error
    );

    throw error;
  }

}