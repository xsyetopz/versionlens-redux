import { ICachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { IHttpOptions } from '#domain/http';
import { DubClient, DubConfig } from '#domain/providers/dub';

export interface IDubServices {

  dubCachingOpts: ICachingOptions;

  dubHttpOpts: IHttpOptions;

  dubConfig: DubConfig;

  dubJsonClient: IJsonHttpClient;

  dubClient: DubClient;

}