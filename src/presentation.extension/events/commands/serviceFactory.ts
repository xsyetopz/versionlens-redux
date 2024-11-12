import { IServiceCollection } from '#domain/di';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import {
  IExtensionServices,
  OnClearCache,
  OnFileLinkClick,
  OnUpdateDependencyClick
} from '#extension';

export function addOnClearCache(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onClearCache;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnClearCache(
        container.packageCache,
        container.shellCache,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnFileLinkClick(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onFileLinkClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) => {
      return new OnFileLinkClick(
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}

export function addOnUpdateDependencyClick(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onUpdateDependencyClick;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) => {
      return new OnUpdateDependencyClick(
        container.versionLensState,
        container.logger.child({ logGroup: serviceName })
      );
    },
    true
  )
}