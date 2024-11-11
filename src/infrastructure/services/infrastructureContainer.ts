import { IServiceCollection } from '#domain/di';
import {
  addPackageFileWatcher,
  addWorkspaceAdapter
} from "#infrastructure/services";

export function addInfrastructureServices(services: IServiceCollection) {

  addWorkspaceAdapter(services);

  addPackageFileWatcher(services);

}