import { IServiceCollection, ServiceInjectionMode } from '#domain/di';
import { ILoggingOptions } from '#domain/logging';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import { OutputChannelTransport, createWinstonLogger } from "infrastructure/logging";
import { PackageFileWatcher, WorkspaceAdapter } from "infrastructure/watcher";
import { workspace } from "vscode";
import { IInfrastructureServices } from "./infrastructureServices";

export function addWorkspaceAdapter(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IInfrastructureServices>().workspaceAdapter,
    () => new WorkspaceAdapter(workspace)
  );
}

export function addWinstonChannelLogger(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDomainServices>().loggerChannel,
    (outputChannel, loggingOptions: ILoggingOptions) =>
      new OutputChannelTransport(
        outputChannel,
        loggingOptions
      ),
    false,
    ServiceInjectionMode.classic
  );
}

export function addWinstonLogger(services: IServiceCollection, defaultLogGroup: string) {
  services.addSingleton(
    nameOf<IDomainServices>().logger,
    (container: IDomainServices) =>
      createWinstonLogger(container.loggerChannel, defaultLogGroup)
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