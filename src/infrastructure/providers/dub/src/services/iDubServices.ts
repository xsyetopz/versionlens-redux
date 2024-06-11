import { ICachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { IHttpOptions } from '#domain/http';
import { DubClient } from '../dubClient';
import { DubConfig } from '../dubConfig';

export interface IDubServices {

  dubCachingOpts: ICachingOptions;

  dubHttpOpts: IHttpOptions;

  dubConfig: DubConfig;

  dubJsonClient: IJsonHttpClient;

  dubClient: DubClient;

}