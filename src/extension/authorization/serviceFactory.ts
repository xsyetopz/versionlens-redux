import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { nameOf } from '#domain/utils';
import type { IExtensionServices } from '#extension';
import {
  AuthenticationInteractions,
  Authorizer,
  UrlAuthenticationStore
} from '#extension/authorization';
import { IVsCodeAuthentication } from '#extension/vscode';
import {
  type Memento,
  type SecretStorage,
  authentication,
  window
} from 'vscode';
import { AuthenticationProviderFactory } from './authenticationProviderFactory';

export function addAuthenticationInteractions(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().authenticationInteractions;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => new AuthenticationInteractions(
      window,
      container.logger.child({ logGroup: serviceName })
    )
  );
}

export function addAuthenticationProviderFactory(
  services: IServiceCollection,
  secrets: SecretStorage
) {
  services.addSingleton(
    nameOf<IExtensionServices>().authenticationProviderFactory,
    (container: IExtensionServices) =>
      new AuthenticationProviderFactory(
        authentication,
        container.authenticationInteractions,
        secrets
      ),
    true
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
        container.authenticationInteractions,
        container.urlAuthenticationStore,
        container.authenticationProviderFactory,
        authentication as IVsCodeAuthentication,
        container.logger.child({ logGroup: serviceName })
      )
  );
}