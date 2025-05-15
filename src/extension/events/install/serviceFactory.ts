import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, type IExtensionServices } from '#extension';
import { OnSaveChanges } from '#extension/events';
import { tasks } from 'vscode';

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
      container.onTextDocumentSave.registerListener(event.execute, event, 2);

      return event;
    }
  )
}