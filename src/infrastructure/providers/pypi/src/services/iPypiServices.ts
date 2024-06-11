import { CachingOptions } from '#domain/caching';
import { IHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { PypiClient } from "../pypiClient";
import { PypiConfig } from "../pypiConfig";

export interface IPypiService {

  pypiCachingOpts: CachingOptions;

  pypiHttpOpts: HttpOptions;

  pypiConfig: PypiConfig;

  pypiHttpClient: IHttpClient;

  pypiClient: PypiClient;

}