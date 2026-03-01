import { DomainServiceName, type IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { LoggerFactory } from '#domain/logging';

/**
 * Registers the logger factory as a singleton in the service collection.
 * @param services The service collection to add to.
 */
export function addLoggerFactory(services: IServiceCollection) {
  services.addSingleton(
    DomainServiceName.loggerFactory,
    (container: IDomainServices) => new LoggerFactory(container.loggerSinks)
  )
}