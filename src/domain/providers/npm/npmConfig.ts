import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { FileMatcher, IProviderConfig } from '#domain/providers';
import { type GitHubOptions, NpmFeatures } from '#domain/providers/npm';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<NpmConfig>();

export class NpmConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: IHttpOptions,
    readonly github: GitHubOptions,
  ) {
    throwUndefinedOrNull(ctorParam.config, config);
    throwUndefinedOrNull(ctorParam.caching, caching);
    throwUndefinedOrNull(ctorParam.http, http);
    throwUndefinedOrNull(ctorParam.github, github);
  }

  get fileMatcher(): FileMatcher {
    return {
      language: 'json',
      scheme: 'file',
      pattern: this.filePatterns
    };
  }

  get filePatterns(): string {
    return this.config.get(NpmFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(NpmFeatures.DependencyProperties);
  }

  get onSaveChangesTask(): string {
    return this.config.get(NpmFeatures.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(NpmFeatures.PrereleaseTagFilter);
  }

}