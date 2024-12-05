import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { nameOf } from '#domain/utils';
import { type IExtensionServices, AuthorizationCommandFeatures } from '#extension';
import { OnAddUrlAuthentication, OnRemoveUrlAuthentication } from '#extension/events';
import { type SecretStorage, commands } from 'vscode';

export function addOnAddUrlAuthentication(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onAddUrlAuthentication;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnAddUrlAuthentication(
        container.authenticationProviders,
        container.urlAuthenticationStore,
        container.packageCache,
        container.authenticationInteractions,
        container.logger.child({ logGroup: serviceName })
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

export function addOnRemoveUrlAuthentication(
  services: IServiceCollection,
  secrets: SecretStorage
) {
  const serviceName = nameOf<IExtensionServices>().onRemoveUrlAuthentication;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      // create the event handler
      const handler = new OnRemoveUrlAuthentication(
        container.urlAuthenticationStore,
        secrets,
        container.packageCache,
        container.authenticationInteractions,
        container.logger.child({ logGroup: serviceName })
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