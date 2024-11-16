import { IServiceCollection } from '#domain/di';
import { addPackageFileWatcher, addWorkspaceAdapter } from './vscode/watcher/serviceFactory';

export function addInfrastructureServices(services: IServiceCollection) {
  // file watcher
  addWorkspaceAdapter(services);
  addPackageFileWatcher(services);
}