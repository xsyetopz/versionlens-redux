import { IServiceCollection } from 'domain/di';
import { IDomainServices } from 'domain/services';
import { nameOf } from 'domain/utils';
import { IExtensionServices } from 'presentation.extension/services/iExtensionServices';
import { OnPackageDependenciesChanged } from './onPackageDependenciesChanged';

export function addOnPackageDependenciesChanged(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onPackageDependenciesChanged
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const event = new OnPackageDependenciesChanged(
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.packageFileWatcher.registerListener(event.execute, event);

      return event;
    }
  )
}