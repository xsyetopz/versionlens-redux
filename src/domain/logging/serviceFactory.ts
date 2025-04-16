import { DomainServiceName, type IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { LoggerFactory } from '#domain/logging';

export function addLoggerFactory(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.loggerFactory,
    (container: IDomainServices) => new LoggerFactory(container.loggerSinks)
  )
}