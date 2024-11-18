import { IServiceCollection } from '#domain/di';
import { addPackageFileWatcher } from './vscode/watcher/serviceFactory';

export function addInfrastructureServices(services: IServiceCollection) {
  // file watcher
  addPackageFileWatcher(services);
}