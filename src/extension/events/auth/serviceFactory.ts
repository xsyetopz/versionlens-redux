import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { AuthorizationCommandFeatures, ExtensionServiceName, type IExtensionServices } from '#extension';
import { OnAddUrlAuthentication, OnRemoveUrlAuthentication } from '#extension/events';
import { commands } from 'vscode';

export function addOnAddUrlAuthentication(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onAddUrlAuthentication;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnAddUrlAuthentication(
        container.authenticationProviders,
        container.urlAuthenticationStore,
        container.packageCache,
        container.authenticationInteractions,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        AuthorizationCommandFeatures.OnAddUrlAuthentication,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}

export function addOnRemoveUrlAuthentication(services: IServiceCollection) {
  const serviceName = ExtensionServiceName.onRemoveUrlAuthentication;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnRemoveUrlAuthentication(
        container.authenticationProviders,
        container.urlAuthenticationStore,
        container.packageCache,
        container.authenticationInteractions,
        container.loggerFactory.create(serviceName)
      );

      // register the vscode command
      handler.disposable = commands.registerCommand(
        AuthorizationCommandFeatures.OnRemoveUrlAuthentication,
        handler.execute,
        handler
      );

      return handler;
    },
    true
  )
}