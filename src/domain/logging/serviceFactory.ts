import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { LoggerFactory } from '#domain/logging';
import { nameOf } from '#domain/utils';

export function addLoggerFactory(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().loggerFactory,
    (container: IDomainServices) => new LoggerFactory(container.loggerSinks)
  )
}