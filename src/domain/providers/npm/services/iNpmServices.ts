import { CachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
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