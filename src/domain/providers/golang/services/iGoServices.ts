import { CachingOptions } from '#domain/caching';
import { HttpOptions, IHttpClient } from '#domain/clients';
import { GoClient, GoConfig } from '#domain/providers/golang';

export interface IGoService {

  goCachingOpts: CachingOptions;

  goHttpOpts: HttpOptions;

  goConfig: GoConfig;

  goHttpClient: IHttpClient;

  goClient: GoClient;

}