import { IServiceCollection } from '#domain/di';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { IExtensionServices, OnTogglePrereleases, OnToggleReleases } from 'presentation.extension';
import { OnErrorClick } from './onErrorClick';

export function addOnErrorClick(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onErrorClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnErrorClick(
        container.extension.state,
        container.outputChannel,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnToggleReleases(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onToggleReleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnToggleReleases(
        container.versionLensProviders,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnTogglePrereleases(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTogglePrereleases;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnTogglePrereleases(
        container.versionLensProviders,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}