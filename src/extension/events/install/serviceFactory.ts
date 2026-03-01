import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { type IExtensionServices, ExtensionServiceName } from '#extension';
import { OnSaveChanges } from '#extension/events';
import { tasks } from 'vscode';

/**
 * Registers the onSaveChanges event handler as a singleton.
 * @param services The service collection to add to.
 */
export function addOnSaveChanges(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnSaveChanges(
        container.fileWatcherDependencyCache,
        container.editorDependencyCache,
        tasks,
        container.extension.state,
        container.loggerFactory.create(serviceName)
      );

      // register listener
      container.onTextDocumentSave.registerListener(event.execute, event);

      return event;
    }
  )
}