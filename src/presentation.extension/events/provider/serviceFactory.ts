import { IServiceCollection } from '#domain/di';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { IExtensionServices } from '#extension';
import {
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose
} from '#extension/events';

export function addOnProviderEditorActivated(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderEditorActivated;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnProviderEditorActivated(
        container.loggerChannel,
        container.extension,
        container.packageFileWatcher,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onActiveTextEditorChange.registerListener(event.execute, event);

      return event;
    },
    false
  )
}

export function addOnProviderTextDocumentChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnProviderTextDocumentChange(
        container.extension.state,
        container.getDependencyChanges,
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentChange.registerListener(event.execute, event);

      return event;
    },
    false
  )
}

export function addOnProviderTextDocumentClose(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderTextDocumentClose
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnProviderTextDocumentClose(
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentClose.registerListener(event.execute, event);

      return event;
    }
  )
}