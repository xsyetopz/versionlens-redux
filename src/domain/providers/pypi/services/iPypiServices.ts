import { CachingOptions } from '#domain/caching';
import { HttpOptions, IHttpClient } from '#domain/clients';
import { PypiClient, PypiConfig } from '#domain/providers/pypi';

export interface IPypiService {

  pypiCachingOpts: CachingOptions;

  pypiHttpOpts: HttpOptions;

  pypiConfig: PypiConfig;

  pypiHttpClient: IHttpClient;

  pypiClient: PypiClient;

}