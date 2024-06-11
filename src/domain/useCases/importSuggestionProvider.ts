import { IServiceCollectionFactory, IServiceProvider } from '#domain/di';
import { ILogger } from 'domain/logging';
import { IProviderModule, ISuggestionProvider } from 'domain/providers';
import { IDomainServices, IProviderServices } from 'domain/services';
import { nameOf } from 'domain/utils';

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
    const module: IProviderModule = await import(
      `../../infrastructure/providers/${providerName}/src/index.ts`
    );

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
      suggestionProvider.config.fileMatcher.pattern,
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