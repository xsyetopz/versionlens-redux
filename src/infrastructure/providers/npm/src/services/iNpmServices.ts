import { CachingOptions } from '#domain/caching';
import { IJsonHttpClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
import { GitHubClient } from '../clients/githubClient';
import { NpmPackageClient } from '../clients/npmPackageClient';
import { NpmRegistryClient } from '../clients/npmRegistryClient';
import { NpmConfig } from '../npmConfig';
import { GitHubOptions } from '../options/githubOptions';

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