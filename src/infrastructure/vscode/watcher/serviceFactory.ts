import type { IServiceCollection } from '#domain/di';
import type { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import type { IInfrastructureServices } from '#infrastructure';
import { PackageFileWatcher, VsCodeWorkspace } from '#infrastructure/vscode';
import { workspace } from 'vscode';

export function addWorkspaceAdapter(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IInfrastructureServices>().workspaceAdapter,
    () => new VsCodeWorkspace(workspace)
  );
}

export function addPackageFileWatcher(services: IServiceCollection) {
  const serviceName = nameOf<IDomainServices>().packageFileWatcher;
  services.addSingleton(
    serviceName,
    (container: IInfrastructureServices & IDomainServices) =>
      new PackageFileWatcher(
        container.getDependencyChanges,
        container.workspaceAdapter,
        container.suggestionProviders,
        container.fileWatcherDependencyCache,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  );
}