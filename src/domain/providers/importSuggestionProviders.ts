import type { IDomainServices } from '#domain';
import type { IServiceCollectionFactory, IServiceProvider } from '#domain/di';
import type { ILogger } from '#domain/logging';
import type {
  IProviderModule,
  IProviderServices,
  ISuggestionProvider
} from '#domain/providers';
import { nameOf } from '#domain/utils';

export function importSuggestionProviders(
  serviceProvider: IServiceProvider,
  providerNames: Array<string>,
  logger: ILogger
): Promise<Array<ISuggestionProvider>> {

  logger.debug('Loading suggestion providers %o', providerNames.join(', '));

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

export async function importSuggestionProvider(
  serviceProvider: IServiceProvider,
  providerName: string,
  logger: ILogger
): Promise<ISuggestionProvider> {

  try {

    logger.debug('Activating container scope for %s', providerName);

    // get the service collection factory
    const serviceCollectionFactory = serviceProvider.getService<IServiceCollectionFactory>(
      nameOf<IDomainServices>().serviceCollectionFactory
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

    logger.debug(
      "Registered provider for %s:\t file pattern: %s\t caching: %s seconds\t strict ssl: %s",
      providerName,
      suggestionProvider.config.filePatterns,
      suggestionProvider.config.caching.duration / 1000,
      suggestionProvider.config.http.strictSSL,
    );

    return suggestionProvider;

  } catch (error) {

    logger.error(
      'Could not register provider %s. Reason: %O',
      providerName,
      error,
    );

    throw error;
  }

}