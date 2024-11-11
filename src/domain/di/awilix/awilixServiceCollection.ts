import {
  IServiceCollection,
  IServiceProvider,
  ServiceInjectionMode,
  ServiceLifetime,
  TServiceResolver
} from '#domain/di';
import { AwilixServiceProvider, registerAsyncSingletons } from '#domain/di/awilix';
import { IDomainServices } from '#domain/services';
import { AsyncFunction, IDisposable, KeyDictionary, nameOf } from '#domain/utils';
import {
  asFunction,
  asValue,
  AwilixContainer,
  createContainer
} from 'awilix';

export class AwilixServiceCollection implements IServiceCollection {

  private resolvers: KeyDictionary<TServiceResolver<any>> = {};

  private asyncSingletons: KeyDictionary<any> = {};

  addSingleton<T>(
    name: string,
    resolver: TServiceResolver<T>,
    isDisposable: boolean = false,
    injectionMode: ServiceInjectionMode = ServiceInjectionMode.proxy
  ): IServiceCollection {
    this.add(name, resolver, ServiceLifetime.singleton, isDisposable, injectionMode);
    return this;
  }

  addScoped<T>(
    name: string,
    resolver: TServiceResolver<T>,
    isDisposable: boolean = false,
    injectionMode: ServiceInjectionMode = ServiceInjectionMode.proxy
  ): IServiceCollection {
    this.add(name, resolver, ServiceLifetime.scoped, isDisposable, injectionMode);
    return this;
  }

  build(): Promise<IServiceProvider> {
    const container: AwilixContainer<any> = createContainer();
    return this.buildAwilixContainer("root", container);
  }

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

  private async buildAwilixContainer(
    name: string,
    container: AwilixContainer<any>
  ): Promise<IServiceProvider> {

    // add the service provider to the container
    const serviceProvider = new AwilixServiceProvider(name, container);
    this.addSingleton(
      nameOf<IDomainServices>().serviceProvider,
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