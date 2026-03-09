import type { ILogger } from '#domain/logging';
import { SuggestionStatusText } from '#domain/packages';
import {
  RubyConfig,
  RubyGitHubClient,
  RubyHttpClient,
  RubySuggestionResolver
} from '#domain/providers/ruby';
import { equal } from 'node:assert';
import { anything, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  configMock: RubyConfig
  httpClientMock: RubyHttpClient
  githubClientMock: RubyGitHubClient
  loggerMock: ILogger
  cut: RubySuggestionResolver
}

export const RubySuggestionResolverTests = {

  title: RubySuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<RubyConfig>();
    this.httpClientMock = mock<RubyHttpClient>();
    this.githubClientMock = mock<RubyGitHubClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new RubySuggestionResolver(
      instance(this.configMock),
      instance(this.httpClientMock),
      instance(this.githubClientMock),
      instance(this.loggerMock)
    );

    when(this.configMock.prereleaseTagFilter).thenReturn([])
  },

  'fromGit: resolves commits when no ref is provided': async function (this: TestContext) {
    const testUser = 'rails';
    const testProject = 'rails';
    const testUrl = 'https://github.com/rails/rails.git';
    const testRef = '';

    const testResponse: any = {
      source: 'github',
      type: 'committish',
      suggestions: []
    };

    when(this.githubClientMock.fetchCommits(testUser, testProject, testRef))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fromGitHub(testUrl, testRef)

    // assert
    equal(actual, testResponse)
    verify(this.githubClientMock.fetchCommits(testUser, testProject, testRef)).once()
  },

  'fromGit: resolves tags when isTag is true': async function (this: TestContext) {
    const testUser = 'rails';
    const testProject = 'rails';
    const testUrl = 'github:rails/rails';
    const testTag = 'v6.0.0';
    const isTag = true;

    const testResponse: any = {
      source: 'github',
      type: 'committish',
      suggestions: [{ name: 'Fixed', version: 'v6.0.0' }]
    };

    when(this.githubClientMock.fetchTags(testUser, testProject, testTag, anything()))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fromGitHub(testUrl, testTag, isTag)

    // assert
    equal(actual, testResponse)
    verify(this.githubClientMock.fetchTags(testUser, testProject, testTag, anything())).once()
  },

  'fromGit: resolves commits when isTag is false': async function (this: TestContext) {
    const testUser = 'rails';
    const testProject = 'rails';
    const testUrl = `github:${testUser}/${testProject}`;
    const testRef = 'a1b2c3d';
    const isTag = false;

    const testResponse: any = {
      source: 'github',
      type: 'committish',
      suggestions: []
    };

    when(this.githubClientMock.fetchCommits(testUser, testProject, testRef))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fromGitHub(testUrl, testRef, isTag)

    // assert
    equal(actual, testResponse)
    verify(this.githubClientMock.fetchCommits(testUser, testProject, testRef)).once()
  },

  'fromGit: doesn\'t fall back to commits when isTag is true': async function (this: TestContext) {
    const testUser = 'rails';
    const testProject = 'rails';
    const testUrl = `git@github.com:${testUser}/${testProject}.git`;
    const testRef = 'abcdef';
    const isTag = true;

    const tagResponse: any = {
      suggestions: [{ name: SuggestionStatusText.NoMatch }]
    };

    when(this.githubClientMock.fetchTags(testUser, testProject, testRef, anything()))
      .thenResolve(tagResponse)

    // test
    const actual = await this.cut.fromGitHub(testUrl, testRef, isTag)

    // assert
    equal(actual, tagResponse)
    verify(this.githubClientMock.fetchTags(testUser, testProject, testRef, anything())).once()
    verify(this.githubClientMock.fetchCommits(anything(), anything(), anything())).never()
  },

}
