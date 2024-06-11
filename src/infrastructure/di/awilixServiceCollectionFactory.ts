import { IServiceCollection, IServiceCollectionFactory } from '#domain/di';
import { AwilixServiceCollection } from ".";

export class AwilixServiceCollectionFactory implements IServiceCollectionFactory {

  createServiceCollection(): IServiceCollection {
    return new AwilixServiceCollection();
  }

}