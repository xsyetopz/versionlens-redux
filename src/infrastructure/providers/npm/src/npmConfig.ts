import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { nameOf } from '#domain/utils';
import { NpmContributions } from './definitions/eNpmContributions';
import { GitHubOptions } from './options/githubOptions';

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
    return this.config.get(NpmContributions.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(NpmContributions.DependencyProperties);
  }

  get onSaveChangesTask(): string {
    return this.config.get(NpmContributions.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(NpmContributions.PrereleaseTagFilter);
  }

}