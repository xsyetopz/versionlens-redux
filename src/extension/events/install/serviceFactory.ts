import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { ExtensionServiceName, type IExtensionServices } from '#extension';
import { OnPreSaveChanges, OnSaveChanges } from '#extension/events';
import { tasks } from 'vscode';

export function addOnPreSaveChanges(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onPreSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnPreSaveChanges(
        container.fileWatcherDependencyCache,
        container.editorDependencyCache,
        container.loggerFactory.create(serviceName)
      );

      // register listener
      container.onTextDocumentSave.registerListener(event.execute, event, 1);

      return event;
    }
  )
}

export function addOnSaveChanges(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onSaveChanges
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnSaveChanges(
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