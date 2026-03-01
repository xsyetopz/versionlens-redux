import { IDisposable } from '#domain/utils';

/**
 * Function that resolves a service instance.
 */
type TServiceResolverFunction<T> = (...args: Array<any>) => T;

/**
 * A static value used as a service instance.
 */
type TServiceResolverValue<T> = T;

/**
 * An asynchronous function that resolves a service instance.
 */
export type TServiceResolverAsyncFunction<T> = Promise<TServiceResolverFunction<Promise<T>>>

/**
 * Union type representing all possible service resolver types.
 */
export type TServiceResolver<T> = TServiceResolverFunction<T>
  | TServiceResolverAsyncFunction<T>
  | TServiceResolverValue<T>

/**
 * Enum representing the lifetime of a registered service.
 */
export enum ServiceLifetime {
  /** A single instance is created for the entire application. */
  singleton = "singleton",
  /** A new instance is created for each scope (e.g., per request or per provider). */
  scoped = "scoped",
  /** A new instance is created every time the service is requested. */
  transient = "transient"
}

/**
 * Enum representing how dependencies are injected into service factories.
 */
export enum ServiceInjectionMode {
  /** Dependencies are passed as individual arguments to the factory function. */
  classic = "classic",
  /** Dependencies are passed as a single object (proxy) to the factory function. */
  proxy = "proxy"
}

/**
 * Interface for a collection of service registrations.
 */
export interface IServiceCollection {

  /**
   * Registers a singleton service.
   * @param name The unique name of the service.
   * @param descriptor The resolver or value for the service.
   * @param isDisposable Whether the service should be disposed of when the container is disposed.
   * @param injectionMode The injection mode to use.
   * @returns The service collection for chaining.
   */
  addSingleton: <T>(
    name: string,
    descriptor: TServiceResolver<T>,
    isDisposable?: boolean,
    injectionMode?: ServiceInjectionMode
  ) => IServiceCollection;

  /**
   * Registers a scoped service.
   * @param name The unique name of the service.
   * @param descriptor The resolver or value for the service.
   * @param isDisposable Whether the service should be disposed of when the scope is disposed.
   * @param injectionMode The injection mode to use.
   * @returns The service collection for chaining.
   */
  addScoped: <T>(
    name: string,
    descriptor: TServiceResolver<T>,
    isDisposable?: boolean,
    injectionMode?: ServiceInjectionMode
  ) => IServiceCollection;

  /**
   * Builds the service provider from the registered services.
   * @returns A promise resolving to the service provider.
   */
  build: () => Promise<IServiceProvider>;

  /**
   * Builds a child service provider with its own scope.
   * @param name The name of the child scope.
   * @param serviceProvider The parent service provider.
   * @returns A promise resolving to the child service provider.
   */
  buildChild(name: string, serviceProvider: IServiceProvider): Promise<IServiceProvider>;

}

/**
 * Interface for a factory that creates service collections.
 */
export interface IServiceCollectionFactory {

  /**
   * Creates a new service collection.
   */
  createServiceCollection: () => IServiceCollection

}

/**
 * Interface for a service provider that can resolve registered services.
 */
export interface IServiceProvider extends IDisposable {

  /** The name of the service provider or scope. */
  name: string;

  /**
   * Manually registers a service instance.
   * @param name The unique name of the service.
   * @param resolver The service instance.
   */
  registerService<T>(name: string, resolver: T): void;

  /**
   * Resolves a service by name.
   * @param name The unique name of the service.
   * @returns The resolved service instance.
   */
  getService<T>(name: string): T;

}