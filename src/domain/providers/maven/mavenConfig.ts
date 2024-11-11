import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { MavenFeatures } from '#domain/providers/maven';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<MavenConfig>();

export class MavenConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: ICachingOptions,
    readonly http: IHttpOptions
  ) {
    throwUndefinedOrNull(ctorParam.config, config);
    throwUndefinedOrNull(ctorParam.caching, caching);
    throwUndefinedOrNull(ctorParam.http, http);
  }

  get fileMatcher(): TProviderFileMatcher {
    return {
      language: 'xml',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: ''
    };
  }

  get filePatterns(): string {
    return this.config.get(MavenFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(MavenFeatures.DependencyProperties);
  }

  get apiUrl(): string {
    return ensureEndSlash(this.config.get(MavenFeatures.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(MavenFeatures.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(MavenFeatures.prereleaseTagFilter);
  }

}