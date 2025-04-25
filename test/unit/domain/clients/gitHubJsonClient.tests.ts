import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import {
  type IJsonHttpClient,
  type JsonClientResponse,
  ClientResponseSource,
  GitHubJsonClient
} from '#domain/clients';
import { deepEqual } from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';
import Fixtures from './gitHubJsonClient.fixtures';

type TestContext = {
  cachingMock: CachingOptions
  jsonClientMock: IJsonHttpClient
  requestCache: MemoryExpiryCache
  cut: GitHubJsonClient
}

export const GitHubJsonClientTests = {

  title: GitHubJsonClient.name,

  beforeEach: function (this: TestContext) {
    this.cachingMock = mock<CachingOptions>();
    this.jsonClientMock = mock<IJsonHttpClient>();
    this.requestCache = new MemoryExpiryCache('test-cache');
    this.cut = new GitHubJsonClient(
      instance(this.cachingMock),
      instance(this.jsonClientMock),
      this.requestCache
    );
  },

  "returns tags": async function (this: TestContext) {
    const testUser = 'octokit'
    const testProject = 'core.js'
    const testUrl = `https://api.github.com/repos/${testUser}/${testProject}/tags`
    const testResponse: JsonClientResponse<any> = {
      status: 200,
      data: Fixtures.tags.test,
      source: ClientResponseSource.remote
    };
    const expectedResponse: JsonClientResponse<any> = {
      status: 200,
      data: Fixtures.tags.expected,
      source: ClientResponseSource.remote
    };

    when(this.jsonClientMock.get(testUrl, anything(), anything()))
      .thenResolve(testResponse);

    // test
    const actual = await this.cut.getTags(testUser, testProject);

    // assert
    deepEqual(actual, expectedResponse)
    deepEqual(this.requestCache.get(testUrl, 3000), expectedResponse)
  },

  "returns commits": async function (this: TestContext) {
    const testUser = 'octokit'
    const testProject = 'core.js'
    const testUrl = `https://api.github.com/repos/${testUser}/${testProject}/commits`
    const testResponse: JsonClientResponse<any> = {
      status: 200,
      data: Fixtures.commits.test,
      source: ClientResponseSource.remote
    };
    const expectedResponse: JsonClientResponse<any> = {
      status: 200,
      data: Fixtures.commits.expected,
      source: ClientResponseSource.remote
    };

    when(this.jsonClientMock.get(testUrl, anything(), anything()))
      .thenResolve(testResponse);

    // test
    const actual = await this.cut.getCommits(testUser, testProject);

    // assert
    deepEqual(actual, expectedResponse)
    deepEqual(this.requestCache.get(testUrl, 3000), expectedResponse)
  }

}