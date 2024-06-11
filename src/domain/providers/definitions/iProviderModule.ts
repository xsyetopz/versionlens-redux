import { IServiceCollection, IServiceProvider } from '#domain/di';

export interface IProviderModule {

  configureContainer(
    serviceProvider: IServiceProvider,
    services: IServiceCollection
  ): Promise<IServiceProvider>

}