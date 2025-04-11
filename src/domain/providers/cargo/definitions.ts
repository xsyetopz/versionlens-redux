import { CachingOptions } from '#domain/caching';
import { HttpOptions, IJsonHttpClient } from '#domain/clients';
import { CargoConfig, CratesClient } from "#domain/providers/cargo";

export enum CargoFeatures {
  Caching = 'cargo.caching',
  Http = 'cargo.http',
  DependencyProperties = 'cargo.dependencyProperties',
  ApiUrl = 'cargo.apiUrl',
  FilePatterns = 'cargo.files',
  OnSaveChangesTask = 'cargo.onSaveChanges',
  PrereleaseTagFilter = 'cargo.prereleaseTagFilter',
}

export interface ICratesApiItem {
  versions: [{
    num: string,
    yanked: boolean
  }];
}

export interface ICargoService {
  cargoCachingOpts: CachingOptions;
  cargoHttpOpts: HttpOptions;
  cargoConfig: CargoConfig;
  cargoJsonClient: IJsonHttpClient;
  cratesClient: CratesClient;
}