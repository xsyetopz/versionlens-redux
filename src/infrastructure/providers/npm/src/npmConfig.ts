import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { nameOf } from '#domain/utils';
import { GitHubOptions, NpmFeatures } from '#providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<NpmConfig>();

export class NpmConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: ICachingOptions,
    readonly http: IHttpOptions,
    readonly github: GitHubOptions,
  ) {
    throwUndefinedOrNull(ctorParam.config, config);
    throwUndefinedOrNull(ctorParam.caching, caching);
    throwUndefinedOrNull(ctorParam.http, http);
    throwUndefinedOrNull(ctorParam.github, github);
  }

  get fileMatcher(): TProviderFileMatcher {
    return {
      language: 'json',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: '**/node_modules/**'
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