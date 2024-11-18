import type { IServiceCollection } from '#domain/di';
import type { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { PackageFileWatcher } from '#infrastructure/vscode';
import { workspace } from 'vscode';

export function addPackageFileWatcher(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().packageFileWatcher;
  services.addSingleton(
    serviceName,
    (container: IDomainServices) =>
      new PackageFileWatcher(
        container.getDependencyChanges,
        <any>workspace,
        container.suggestionProviders,
        container.fileWatcherDependencyCache,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  );
}