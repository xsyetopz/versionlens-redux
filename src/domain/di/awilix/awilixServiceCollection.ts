import { DomainServiceName } from '#domain';
import {
  type IServiceCollection,
  type IServiceProvider,
  type TServiceResolver,
  ServiceInjectionMode,
  ServiceLifetime
} from '#domain/di';
import { AwilixServiceProvider, registerAsyncSingletons } from '#domain/di/awilix';
import {
  type IDisposable,
  type KeyDictionary,
  AsyncFunction
} from '#domain/utils';
import {
  type AwilixContainer,
  asFunction,
  asValue,
  createContainer
} from 'awilix';

/**
 * Awilix-based implementation of the IServiceCollection interface.
 */
export class AwilixServiceCollection implements IServiceCollection {

  /** Map of registered Awilix resolvers. */
  private resolvers: KeyDictionary<TServiceResolver<any>> = {};

  /** Map of registered asynchronous singletons. */
  private asyncSingletons: KeyDictionary<any> = {};

  /**
   * Registers a singleton service.
   * @param name The unique name of the service.
   * @param resolver The resolver or value for the service.
   * @param isDisposable Whether the service should be disposed of when the container is disposed.
   * @param injectionMode The injection mode to use.
   * @returns The service collection for chaining.
   */
  addSingleton<T>(
    name: string,
    resolver: TServiceResolver<T>,
    isDisposable: boolean = false,
    injectionMode: ServiceInjectionMode = ServiceInjectionMode.proxy
  ): IServiceCollection {
    this.add(name, resolver, ServiceLifetime.singleton, isDisposable, injectionMode);
    return this;
  }

  /**
   * Registers a scoped service.
   * @param name The unique name of the service.
   * @param resolver The resolver or value for the service.
   * @param isDisposable Whether the service should be disposed of when the scope is disposed.
   * @param injectionMode The injection mode to use.
   * @returns The service collection for chaining.
   */
  addScoped<T>(
    name: string,
    resolver: TServiceResolver<T>,
    isDisposable: boolean = false,
    injectionMode: ServiceInjectionMode = ServiceInjectionMode.proxy
  ): IServiceCollection {
    this.add(name, resolver, ServiceLifetime.scoped, isDisposable, injectionMode);
    return this;
  }

  /**
   * Builds the root service provider.
   * @returns A promise resolving to the service provider.
   */
  build(): Promise<IServiceProvider> {
    const container: AwilixContainer<any> = createContainer();
    return this.buildAwilixContainer("root", container);
  }

  /**
   * Builds a child service provider with its own scope.
   * @param name The name of the child scope.
   * @param serviceProvider The parent service provider.
   * @returns A promise resolving to the child service provider.
   */
  async buildChild(
    name: string,
    serviceProvider: IServiceProvider
  ): Promise<IServiceProvider> {
    const container: AwilixContainer<any> = (<any>serviceProvider).container;
    const childContainer = container.createScope();
    return await this.buildAwilixContainer(
      name,
      childContainer
    );
  }

  /**
   * Internal method to add a service registration.
   */
  private add<T>(
    name: string,
    resolver: TServiceResolver<T>,
    lifetime: ServiceLifetime,
    isDisposable: boolean,
    injectionMode: ServiceInjectionMode
  ): IServiceCollection {
    let awilixResolver: any;

    if (resolver instanceof AsyncFunction) {
      this.asyncSingletons[name] = resolver;
      return this;
    }

    if (resolver instanceof Function) {
      awilixResolver = asFunction(resolver)[lifetime]()[injectionMode]();
      if (isDisposable) {
        awilixResolver = awilixResolver.disposer(
          (service: IDisposable) => service.dispose()
        );
      }

    } else {
      awilixResolver = asValue(resolver);
    }

    this.resolvers[name] = awilixResolver;

    return this;
  }

  /**
   * Internal method to configure and build the Awilix container.
   */
  private async buildAwilixContainer(
    name: string,
    container: AwilixContainer<any>
  ): Promise<IServiceProvider> {

    // add the service provider to the container
    const serviceProvider = new AwilixServiceProvider(name, container);
    this.addSingleton(
      DomainServiceName.serviceProvider,
      serviceProvider,
      true
    );

    // register sync
    container.register(this.resolvers);

    // register async
    container.register(
      await registerAsyncSingletons(
        container,
        this.asyncSingletons
      )
    );

    return serviceProvider;
  }

}