import { CachingOptions } from '#domain/caching';
import { IHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { PypiClient, PypiConfig } from '#domain/providers/pypi';

export interface IPypiService {

  pypiCachingOpts: CachingOptions;

  pypiHttpOpts: HttpOptions;

  pypiConfig: PypiConfig;

  pypiHttpClient: IHttpClient;

  pypiClient: PypiClient;

}