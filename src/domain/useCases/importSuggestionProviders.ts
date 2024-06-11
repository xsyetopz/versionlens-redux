import { IServiceProvider } from '#domain/di';
import { ILogger } from 'domain/logging';
import { ISuggestionProvider } from 'domain/providers';
import { importSuggestionProvider } from './importSuggestionProvider';

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