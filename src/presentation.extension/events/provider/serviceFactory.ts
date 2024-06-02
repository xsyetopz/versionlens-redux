import { IServiceCollection } from 'domain/di';
import { IDomainServices } from 'domain/services';
import { nameOf } from 'domain/utils';
import { IExtensionServices } from 'presentation.extension';
import { OnProviderEditorActivated } from './onProviderEditorActivated';
import { OnProviderTextDocumentChange } from './onProviderTextDocumentChange';
import { OnProviderTextDocumentClose } from './onProviderTextDocumentClose';

export function addOnProviderEditorActivated(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderEditorActivated;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const listener = new OnProviderEditorActivated(
        container.loggerChannel,
        container.extension,
        container.packageFileWatcher,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onActiveTextEditorChange.registerListener(listener.execute, listener);
      return listener;
    },
    false
  )
}

export function addOnProviderTextDocumentChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      const listener = new OnProviderTextDocumentChange(
        container.extension.state,
        container.getDependencyChanges,
        container.editorDependencyCache,
        container.logger.child({ logGroup: serviceName })
      );

      // register listener
      container.onTextDocumentChange.registerListener(listener.execute, listener);
      return listener;
    },
    false
  )
}

export function addOnProviderTextDocumentClose(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onProviderTextDocumentClose
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
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