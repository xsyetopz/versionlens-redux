import { IServiceCollection } from "#domain/di";

export interface IServiceCollectionFactory {

  createServiceCollection: () => IServiceCollection

}