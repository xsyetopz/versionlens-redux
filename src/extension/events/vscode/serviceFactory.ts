import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { nameOf } from '#domain/utils';
import type { IExtensionServices } from '#extension';
import {
  OnActiveTextEditorChange,
  OnTextDocumentChange,
  OnTextDocumentClose,
  OnTextDocumentSave
} from '#extension/events';
import { window, workspace } from 'vscode';

export function addOnActiveTextEditorChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onActiveTextEditorChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnActiveTextEditorChange(
        container.extension.state,
        container.GetSuggestionProvider,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode editor event
      event.disposable = window.onDidChangeActiveTextEditor(event.execute, event);

      return event;
    },
    true
  )
}

export function addOnTextDocumentChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnTextDocumentChange(
        container.GetSuggestionProvider,
        container.versionLensState,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode workspace event
      event.disposable = workspace.onDidChangeTextDocument(event.execute, event);

      return event;
    },
    true
  )
}

export function addOnTextDocumentClose(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentClose;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnTextDocumentClose(
        container.GetSuggestionProvider,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode workspace event
      event.disposable = workspace.onDidCloseTextDocument(event.execute, event);

      return event;
    },
    true
  )
}

export function addOnTextDocumentSave(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentSave;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const event = new OnTextDocumentSave(
        container.GetSuggestionProvider,
        container.extension.state,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode workspace event
      event.disposable = workspace.onDidSaveTextDocument(event.execute, event);

      return event;
    },
    true
  )
}