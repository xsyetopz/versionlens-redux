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
  jsonClientMock: IJsonHttpClient
}

export const GitHubJsonClientTests = {

  title: GitHubJsonClient.name,

  beforeEach: function (this: TestContext) {
    this.jsonClientMock = mock<IJsonHttpClient>();
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
    const cut = new GitHubJsonClient(instance(this.jsonClientMock));

    when(this.jsonClientMock.get(testUrl, anything(), anything()))
      .thenResolve(testResponse);

    // test
    const actual = await cut.getTags(testUser, testProject);

    // assert
    deepEqual(actual.data, Fixtures.tags.expected)
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
    const cut = new GitHubJsonClient(instance(this.jsonClientMock));

    when(this.jsonClientMock.get(testUrl, anything(), anything()))
      .thenResolve(testResponse);

    // test
    const actual = await cut.getCommits(testUser, testProject);

    // assert
    deepEqual(actual.data, Fixtures.commits.expected)
  }

}