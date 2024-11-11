import { CachingOptions } from '#domain/caching';
import { HttpOptions, IJsonHttpClient } from '#domain/clients';
import {
  GitHubClient,
  GitHubOptions,
  NpmConfig,
  NpmPackageClient,
  NpmRegistryClient
} from '#domain/providers/npm';

export interface INpmServices {

  npmCachingOpts: CachingOptions;

  npmHttpOpts: HttpOptions;

  npmGitHubOpts: GitHubOptions;

  npmConfig: NpmConfig;

  githubJsonClient: IJsonHttpClient;

  githubClient: GitHubClient;

  npmRegistryClient: NpmRegistryClient;

  npmClient: NpmPackageClient;

}