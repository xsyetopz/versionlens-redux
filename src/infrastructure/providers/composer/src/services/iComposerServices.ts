import { CachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { ComposerClient } from "../composerClient";
import { ComposerConfig } from "../composerConfig";

export interface IComposerService {

  composerCachingOpts: CachingOptions;

  composerHttpOpts: HttpOptions;

  composerConfig: ComposerConfig;

  composerJsonClient: IJsonHttpClient;

  composerClient: ComposerClient;

}