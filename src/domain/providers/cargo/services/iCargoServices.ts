import { CachingOptions } from '#domain/caching';
import { HttpOptions, IJsonHttpClient } from '#domain/clients';
import { CargoConfig, CratesClient } from "#domain/providers/cargo";

export interface ICargoService {

  cargoCachingOpts: CachingOptions;

  cargoHttpOpts: HttpOptions;

  cargoConfig: CargoConfig;

  cargoJsonClient: IJsonHttpClient;

  cratesClient: CratesClient;

}