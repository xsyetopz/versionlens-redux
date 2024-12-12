import type { IDomainServices } from '#domain';
import type { IServiceCollection } from '#domain/di';
import { nameOf } from '#domain/utils';
import type { IExtensionServices } from '#extension';
import { PackageFileWatcher } from '#extension/watcher';
import { workspace } from 'vscode';

export function addPackageFileWatcher(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().packageFileWatcher;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new PackageFileWatcher(
        container.getDependencyChanges,
        container.suggestionProviders,
        container.fileWatcherDependencyCache,
        container.editorConfig,
        workspace,
        container.loggerFactory.create(serviceName)
      ),
    true
  );
}