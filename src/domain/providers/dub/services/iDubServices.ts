import { ICachingOptions } from '#domain/caching';
import { IHttpOptions, IJsonHttpClient } from '#domain/clients';
import { DubClient, DubConfig } from '#domain/providers/dub';

export interface IDubServices {

  dubCachingOpts: ICachingOptions;

  dubHttpOpts: IHttpOptions;

  dubConfig: DubConfig;

  dubJsonClient: IJsonHttpClient;

  dubClient: DubClient;

}