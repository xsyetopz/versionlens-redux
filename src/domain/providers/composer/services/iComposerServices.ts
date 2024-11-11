import { CachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { ComposerClient, ComposerConfig } from "#domain/providers/composer";

export interface IComposerService {

  composerCachingOpts: CachingOptions;

  composerHttpOpts: HttpOptions;

  composerConfig: ComposerConfig;

  composerJsonClient: IJsonHttpClient;

  composerClient: ComposerClient;

}