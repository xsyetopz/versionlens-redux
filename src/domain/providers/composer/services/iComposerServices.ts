import { CachingOptions } from '#domain/caching';
import { HttpOptions, IJsonHttpClient } from '#domain/clients';
import { ComposerClient, ComposerConfig } from "#domain/providers/composer";

export interface IComposerService {

  composerCachingOpts: CachingOptions;

  composerHttpOpts: HttpOptions;

  composerConfig: ComposerConfig;

  composerJsonClient: IJsonHttpClient;

  composerClient: ComposerClient;

}