import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { nameOf } from '#domain/utils';
import type { IExtensionServices } from '#extension';
import {
  AuthenticationInteractions,
  AuthenticationScheme,
  Authorizer,
  BasicAuthProvider,
  CustomAuthProvider,
  UrlAuthenticationStore
} from '#extension/authorization';
import { type Memento, type SecretStorage, window } from 'vscode';

export function addAuthenticationProviders(
  services: IServiceCollection,
  resourceFolderPath: string,
  secrets: SecretStorage
) {
  const serviceName = nameOf<IExtensionServices>().authenticationProviders;
  services.addSingleton(
    serviceName,
    (container: IExtensionServices) => ({
      [AuthenticationScheme.Basic]: new BasicAuthProvider(
        resourceFolderPath,
        secrets,
        container.authenticationInteractions
      ),
      [AuthenticationScheme.Custom]: new CustomAuthProvider(
        resourceFolderPath,
        secrets,
        container.authenticationInteractions
      )
    })
  );
}

export function addAuthenticationInteractions(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().authenticationInteractions;
  services.addSingleton(
    serviceName,
    () => new AuthenticationInteractions(window)
  );
}

export function addUrlAuthenticationStore(
  services: IServiceCollection,
  workspaceState: Memento
) {
  const serviceName = nameOf<IExtensionServices>().urlAuthenticationStore;
  services.addSingleton(
    serviceName,
    () => new UrlAuthenticationStore('UrlAuthenticationStore', workspaceState)
  );
}

export function addAuthorizer(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().authorizer;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new Authorizer(
        container.urlAuthenticationStore,
        container.authenticationProviders,
        container.authenticationInteractions,
        container.logger.child({ logGroup: serviceName })
      )
  );
}