import type { IServiceCollection } from '#domain/di';
import type { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import type { IExtensionServices } from '#extension';
import { OnPreSaveChanges, OnSaveChanges } from '#extension/events';

export function addOnPreSaveChanges(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onPreSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnPreSaveChanges(
        container.fileWatcherDependencyCache,
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentSave.registerListener(event.execute, event, 1);

      return event;
    }
  )
}

export function addOnSaveChanges(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnSaveChanges(
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentSave.registerListener(event.execute, event, 2);

      return event;
    }
  )
}