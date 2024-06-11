import { CachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { CargoConfig } from "../cargoConfig";
import { CratesClient } from "../cratesClient";

export interface ICargoService {

  cargoCachingOpts: CachingOptions;

  cargoHttpOpts: HttpOptions;

  cargoConfig: CargoConfig;

  cargoJsonClient: IJsonHttpClient;

  cratesClient: CratesClient;

}