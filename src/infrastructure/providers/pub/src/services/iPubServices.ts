import { CachingOptions } from '#domain/caching';
import { JsonHttpClient } from '#domain/clients';
import { HttpOptions } from "domain/http";
import { PubClient } from "../pubClient";
import { PubConfig } from "../pubConfig";

export interface IPubServices {

  pubCachingOpts: CachingOptions;

  pubHttpOpts: HttpOptions;

  pubConfig: PubConfig;

  pubJsonClient: JsonHttpClient;

  pubClient: PubClient;

}