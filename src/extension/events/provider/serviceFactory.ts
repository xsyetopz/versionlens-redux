import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, type IExtensionServices } from '#extension';
import {
  OnProviderEditorActivated,
  OnProviderTextDocumentChange,
  OnProviderTextDocumentClose
} from '#extension/events';

/**
 * Registers the onProviderEditorActivated event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnProviderEditorActivated(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onProviderEditorActivated;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnProviderEditorActivated(
        container.extension,
        container.packageFileWatcher,
        container.loggerFactory.create(serviceName)
      );

      // register listener
      container.onActiveTextEditorChange.registerListener(event.execute, event);

      return event;
    },
    false
  )
}

/**
 * Registers the onProviderTextDocumentChange event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnProviderTextDocumentChange(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onProviderTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnProviderTextDocumentChange(
        container.extension.state,
        container.getDependencyChanges,
        container.editorDependencyCache,
        container.loggerFactory.create(serviceName)
      );

      // register listener
      container.onTextDocumentChange.registerListener(event.execute, event);

      return event;
    },
    false
  )
}

/**
 * Registers the onProviderTextDocumentClose event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnProviderTextDocumentClose(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onProviderTextDocumentClose
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnProviderTextDocumentClose(
        container.editorDependencyCache,
        container.loggerFactory.create(serviceName)
      );

      // register listener
      container.onTextDocumentClose.registerListener(event.execute, event);

      return event;
    }
  )
}