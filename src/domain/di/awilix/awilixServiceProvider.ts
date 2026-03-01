import { IServiceProvider } from '#domain/di';
import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';
import { asValue, AwilixContainer } from 'awilix';

/**
 * Awilix-based implementation of the IServiceProvider interface.
 */
export class AwilixServiceProvider implements IServiceProvider {

  /**
   * Initializes a new instance of the AwilixServiceProvider class.
   * @param name The name of the service provider or scope.
   * @param container The underlying Awilix container.
   */
  constructor(
    readonly name: string,
    readonly container: AwilixContainer
  ) {
    throwNotStringOrEmpty("name", name);
    throwUndefinedOrNull("container", container);
  }

  /**
   * Registers a service instance directly in the container.
   * @param name The unique name of the service.
   * @param resolver The service instance.
   */
  registerService<T>(name: string, resolver: T) {
    this.container.register(name, asValue(resolver));
  }

  /**
   * Resolves a service by name from the Awilix container.
   * @param name The unique name of the service.
   * @returns The resolved service instance.
   */
  getService<T>(name: string): T {
    return this.container.resolve<T>(name);
  }

  /**
   * Disposes of the Awilix container and all its scoped services.
   * @returns A promise that resolves when disposal is complete.
   */
  dispose() {
    return this.container.dispose();
  }

}