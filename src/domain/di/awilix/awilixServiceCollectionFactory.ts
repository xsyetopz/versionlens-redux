import { IServiceCollection, IServiceCollectionFactory } from '#domain/di';
import { AwilixServiceCollection } from '#domain/di/awilix';

/**
 * Factory for creating Awilix service collections.
 */
export class AwilixServiceCollectionFactory implements IServiceCollectionFactory {

  /**
   * Creates a new Awilix service collection.
   * @returns A new instance of AwilixServiceCollection.
   */
  createServiceCollection(): IServiceCollection {
    return new AwilixServiceCollection();
  }

}