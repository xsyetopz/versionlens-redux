import { CachingOptions } from '#domain/caching';
import { IHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { GoClient, GoConfig } from '#domain/providers/golang';

export interface IGoService {

  goCachingOpts: CachingOptions;

  goHttpOpts: HttpOptions;

  goConfig: GoConfig;

  goHttpClient: IHttpClient;

  goClient: GoClient;

}