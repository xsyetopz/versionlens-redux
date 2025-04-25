import {
  type GitHubJsonClient,
  type JsonClientResponse,
  ClientResponseSource
} from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageSuggestion,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import { type NpaSpec, type NpmConfig, NpmGitHubClient } from '#domain/providers/npm';
import { deepEqual, equal } from 'node:assert';
import npa from 'npm-package-arg';
import { anything, instance, mock, when } from 'ts-mockito';
import { githubFixtures } from './npmGitHubClient.fixtures';

type TestContext = {
  configMock: NpmConfig
  githubJsonClientMock: GitHubJsonClient
  loggerMock: ILogger
}

export const NpmGitHubClientTests = {

  title: NpmGitHubClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<NpmConfig>();
    this.githubJsonClientMock = mock<GitHubJsonClient>();
    this.loggerMock = mock<ILogger>();

    when(this.configMock.prereleaseTagFilter).thenReturn([])
  },

  'returns a #semver:x.x.x. package': async function (this: TestContext) {
    const testRequest: any = {
      providerName: 'testnpmprovider',
      package: {
        path: 'packagepath',
        name: 'core.js',
        version: 'github:octokit/core.js#semver:^2',
      }
    };

    const testSpec: NpaSpec = npa.resolve(
      testRequest.package.name,
      testRequest.package.version,
      testRequest.package.path
    ) as NpaSpec;

    const testResponse: JsonClientResponse<any> = {
      status: 200,
      data: githubFixtures.tags,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getTags(anything(), anything()))
      .thenResolve(testResponse)

    // setup initial call
    const cut = new NpmGitHubClient(
      instance(this.configMock),
      instance(this.githubJsonClientMock),
      instance(this.loggerMock)
    );

    // test
    const actual = await cut.fetchGithub(testSpec)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'range')
    equal(actual.resolved?.name, testRequest.package.name)
    deepEqual(
      actual.suggestions,
      [
        <PackageSuggestion>{
          name: SuggestionStatusText.SatisfiesLatest,
          category: SuggestionCategory.Match,
          version: 'v2.5.0',
          type: SuggestionTypes.status
        },
        <PackageSuggestion>{
          name: SuggestionStatusText.Latest,
          category: SuggestionCategory.Updateable,
          version: 'v2.5.0',
          type: SuggestionTypes.release
        },
        <PackageSuggestion>{
          name: 'rc',
          category: SuggestionCategory.Updateable,
          version: 'v2.6.0-rc.1',
          type: SuggestionTypes.prerelease
        },
        <PackageSuggestion>{
          name: 'preview',
          category: SuggestionCategory.Updateable,
          version: 'v2.5.0-preview.1',
          type: SuggestionTypes.prerelease
        }
      ]
    )
  },

  'returns a #x.x.x': async function (this: TestContext) {

    const testRequest: any = {
      providerName: 'testnpmprovider',
      package: {
        path: 'packagepath',
        name: 'core.js',
        version: 'github:octokit/core.js#v2.0.0',
      }
    };

    const testSpec = npa.resolve(
      testRequest.package.name,
      testRequest.package.version,
      testRequest.package.path
    ) as NpaSpec;

    const testResponse: JsonClientResponse<any> = {
      status: 200,
      data: githubFixtures.tags,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getTags(anything(), anything()))
      .thenResolve(testResponse)

    // setup initial call
    const cut = new NpmGitHubClient(
      instance(this.configMock),
      instance(this.githubJsonClientMock),
      instance(this.loggerMock)
    );

    // test
    const actual = await cut.fetchGithub(testSpec)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'range')
    equal(actual.resolved?.name, testRequest.package.name)
    deepEqual(
      actual.suggestions,
      [
        <PackageSuggestion>{
          name: SuggestionStatusText.Fixed,
          category: SuggestionCategory.Match,
          version: 'v2.0.0',
          type: SuggestionTypes.status
        },
        <PackageSuggestion>{
          name: SuggestionStatusText.UpdateLatest,
          category: SuggestionCategory.Updateable,
          version: 'v2.5.0',
          type: SuggestionTypes.release
        },
        <PackageSuggestion>{
          name: 'rc',
          category: SuggestionCategory.Updateable,
          version: 'v2.6.0-rc.1',
          type: SuggestionTypes.prerelease
        },
        <PackageSuggestion>{
          name: 'preview',
          category: SuggestionCategory.Updateable,
          version: 'v2.5.0-preview.1',
          type: SuggestionTypes.prerelease
        }
      ]
    )
  },

  'returns a #sha commit': async function (this: TestContext) {

    const testRequest: any = {
      providerName: 'testnpmprovider',
      package: {
        path: 'packagepath',
        name: 'core.js',
        version: 'github:octokit/core.js#166c3497',
      }
    };

    const testSpec = npa.resolve(
      testRequest.package.name,
      testRequest.package.version,
      testRequest.package.path
    ) as NpaSpec;

    const testResponse: JsonClientResponse<any> = {
      status: 200,
      data: githubFixtures.commits,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(anything(), anything()))
      .thenResolve(testResponse)

    const cut = new NpmGitHubClient(
      instance(this.configMock),
      instance(this.githubJsonClientMock),
      instance(this.loggerMock)
    );

    // test
    const actual = await cut.fetchGithub(testSpec)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'committish')
    equal(actual.resolved?.name, testRequest.package.name)
    deepEqual(
      actual.suggestions,
      [
        <PackageSuggestion>{
          name: SuggestionStatusText.Fixed,
          category: SuggestionCategory.Match,
          version: '166c3497',
          type: SuggestionTypes.status
        },
        <PackageSuggestion>{
          name: SuggestionStatusText.UpdateLatest,
          category: SuggestionCategory.Updateable,
          version: 'df4d9435',
          type: SuggestionTypes.release
        }
      ]
    )
  },

}