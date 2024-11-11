import { CachingOptions } from '#domain/caching';
import { HttpOptions, JsonHttpClient } from '#domain/clients';
import { PubClient, PubConfig } from '#domain/providers/pub';

export interface IPubServices {

  pubCachingOpts: CachingOptions;

  pubHttpOpts: HttpOptions;

  pubConfig: PubConfig;

  pubJsonClient: JsonHttpClient;

  pubClient: PubClient;

}