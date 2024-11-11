import { IServiceCollection } from '#domain/di';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { IInfrastructureServices } from '#infrastructure/services';
import { PackageFileWatcher, WorkspaceAdapter } from '#infrastructure/watcher';
import { workspace } from "vscode";

export function addWorkspaceAdapter(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IInfrastructureServices>().workspaceAdapter,
    () => new WorkspaceAdapter(workspace)
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