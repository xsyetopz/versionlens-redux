import {
  ClientResponseSource,
  GitHubJsonClient,
  type GithubJsonClientResponse
} from '#domain/clients';
import { PackageStatusFactory, UpdateableFactory } from '#domain/packages';
import { RubyGitHubClient } from '#domain/providers/ruby';
import { deepEqual, equal } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from '../npm/npmGitHubClient.fixtures';

type TestContext = {
  githubJsonClientMock: GitHubJsonClient
  cut: RubyGitHubClient
}

export const GitHubClientTests = {

  title: RubyGitHubClient.name,

  beforeEach: function (this: TestContext) {
    this.githubJsonClientMock = mock<GitHubJsonClient>();
    this.cut = new RubyGitHubClient(
      instance(this.githubJsonClientMock)
    );
  },

  'fetchTags returns suggestions': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testRange = 'v2.5.0';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: Fixtures.tags,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getTags(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchTags(testUser, testProject, testRange)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'range')
    equal(actual.resolved?.name, testProject)
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createMatchesLatestStatus('v2.5.0'),
        UpdateableFactory.createTaggedPreleaseUpdateable('rc', 'v2.6.0-rc.1')
      ]
    )
  },

  'fetchTags returns not found status when range is not in versions': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testRange = 'v9.9.9';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: Fixtures.tags,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getTags(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchTags(testUser, testProject, testRange)

    // assert
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createNotFoundStatus(),
        UpdateableFactory.createLatestUpdateable('v2.5.0')
      ]
    )
  },

  'fetchCommits returns not found response when no commits found': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testSha = '166c349';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: [],
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchCommits(testUser, testProject, testSha)

    // assert
    deepEqual(
      actual.suggestions,
      [PackageStatusFactory.createNotFoundStatus()]
    )
  },

  'fetchCommits returns no match status when commitSha not in commits': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testSha = 'non-existent';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: Fixtures.commits,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchCommits(testUser, testProject, testSha)

    // assert
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createNoMatchStatus(),
        UpdateableFactory.createLatestUpdateable('df4d943')
      ]
    )
  },

  'fetchCommits returns matches latest status when commitSha is latest': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testSha = 'df4d943';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: Fixtures.commits,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchCommits(testUser, testProject, testSha)

    // assert
    deepEqual(
      actual.suggestions,
      [PackageStatusFactory.createMatchesLatestStatus('df4d943')]
    )
  },

  'fetchCommits returns fixed status when commitSha is not latest': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testSha = '166c349';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: Fixtures.commits,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchCommits(testUser, testProject, testSha)

    // assert
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createFixedStatus('166c349'),
        UpdateableFactory.createLatestUpdateable('df4d943')
      ]
    )
  },

  'fetchCommits returns fixed status when commitSha is a long SHA': async function (this: TestContext) {
    const testUser = 'octokit';
    const testProject = 'core.js';
    const testSha = '166c34965efb28e18e4f1f3a1c668748496a5a93';

    const testResponse: GithubJsonClientResponse = {
      status: 200,
      data: Fixtures.commits,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(testUser, testProject))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchCommits(testUser, testProject, testSha)

    // assert
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createFixedStatus('166c349'),
        UpdateableFactory.createLatestUpdateable('df4d943')
      ]
    )
  },

}
