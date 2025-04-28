import type { CachingOptions, IExpiryCache } from '#domain/caching';
import {
  type GithubCommitsApiResult,
  type GithubJsonClientResponse,
  type GithubTagsApiResult,
  type IJsonHttpClient,
  ClientResponseSource
} from '#domain/clients';
import { throwUndefinedOrNull } from '@esm-test/guards';

const defaultHeaders = {
  accept: 'application\/vnd.github.v3+json'
};

export class GitHubJsonClient {

  constructor(
    readonly caching: CachingOptions,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<GithubJsonClientResponse>
  ) {
    throwUndefinedOrNull('caching', caching);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
  }

  async getTags(user: string, project: string): Promise<GithubJsonClientResponse> {
    const tagsRepoUrl = `https://api.github.com/repos/${user}/${project}/tags`;
    // check cache
    const cached = this.requestCache.get(tagsRepoUrl, this.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<GithubTagsApiResult>(
      tagsRepoUrl,
      {},
      defaultHeaders
    );
    // reduce
    const tags = jsonResponse.data ?? [];
    const data = tags.map(tag => tag.name);
    // cache and return
    const result = { ...jsonResponse, data };
    return this.requestCache.set(tagsRepoUrl, result);
  }

  async getCommits(user: string, project: string): Promise<GithubJsonClientResponse> {
    const commitsRepoUrl = `https://api.github.com/repos/${user}/${project}/commits`;
    // check cache
    const cached = this.requestCache.get(
      commitsRepoUrl,
      this.caching.duration
    );
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<GithubCommitsApiResult>(
      commitsRepoUrl,
      {},
      defaultHeaders
    );
    // reduce
    const commits = jsonResponse.data ?? []
    const data = commits.map((commit: any) => commit.sha.substring(0, 7));
    // cache and return
    const result = { ...jsonResponse, data };
    return this.requestCache.set(commitsRepoUrl, result);
  }

}