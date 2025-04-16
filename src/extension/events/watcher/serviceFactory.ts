import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, OnPackageDependenciesChanged, type IExtensionServices } from '#extension';

export function addOnPackageDependenciesChanged(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onPackageDependenciesChanged
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const event = new OnPackageDependenciesChanged(
        container.extension.state,
        container.loggerFactory.create(serviceName)
      );

      // register listener
      container.packageFileWatcher.registerListener(event.execute, event);

      return event;
    }
  )
}