import { IServiceCollection } from "domain/di";
import {
  addPackageFileWatcher,
  addWinstonChannelLogger,
  addWinstonLogger,
  addWorkspaceAdapter
} from ".";

export function addInfrastructureServices(services: IServiceCollection, defaultLogGroup: string) {

  addWorkspaceAdapter(services);

  addWinstonChannelLogger(services);

  addWinstonLogger(services, defaultLogGroup);

  addPackageFileWatcher(services);

}