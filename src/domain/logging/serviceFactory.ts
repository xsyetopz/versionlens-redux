import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { LoggingOptions } from '#domain/logging';
import { createWinstonLogger } from '#domain/logging/winston';
import { nameOf } from '#domain/utils';

export function addLoggingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().loggingOptions,
    (container: IDomainServices) => new LoggingOptions(container.appConfig, 'logging')
  )
}

export function addWinstonLogger(services: IServiceCollection, defaultLogGroup: string) {
  services.addSingleton(
    nameOf<IDomainServices>().logger,
    (container: IDomainServices) =>
      createWinstonLogger(container.loggerChannel, defaultLogGroup)
  );
}