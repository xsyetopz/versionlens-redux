import { IServiceCollection, IServiceCollectionFactory } from '#domain/di';
import { AwilixServiceCollection } from '#domain/di/awilix';

export class AwilixServiceCollectionFactory implements IServiceCollectionFactory {

  createServiceCollection(): IServiceCollection {
    return new AwilixServiceCollection();
  }

}