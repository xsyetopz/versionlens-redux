import {
  type IJsonHttpClient,
  type JsonClientResponse,
  ClientResponseSource,
  JsonHttpClient
} from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageSuggestion,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import {
  type NpaSpec,
  GitHubClient,
  GitHubOptions,
  NpmConfig
} from '#domain/providers/npm';
import { deepEqual, equal, notEqual } from 'node:assert';
import npa from 'npm-package-arg';
import { anything, capture, instance, mock, when } from 'ts-mockito';
import { githubFixtures } from './fetchGitHub.fixtures';

let githubOptsMock: GitHubOptions;
let configMock: NpmConfig;
let loggerMock: ILogger;
let jsonClientMock: IJsonHttpClient;

export const fetchGithubTests = {

  title: GitHubClient.prototype.fetchGithub.name,

  beforeEach: () => {
    githubOptsMock = mock(GitHubOptions);
    configMock = mock(NpmConfig);
    jsonClientMock = mock(JsonHttpClient);
    loggerMock = mock<ILogger>();

    when(configMock.github).thenReturn(instance(githubOptsMock))
    when(configMock.prereleaseTagFilter).thenReturn([])
  },

  'returns a #semver:x.x.x. package': async () => {
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

    const testResponse: JsonClientResponse = {
      status: 200,
      data: githubFixtures.tags,
      source: ClientResponseSource.remote
    };

    when(jsonClientMock.get(anything(), anything(), anything()))
      .thenResolve(testResponse)

    // setup initial call
    const cut = new GitHubClient(
      instance(configMock),
      instance(jsonClientMock),
      instance(loggerMock)
    );

    return cut.fetchGithub(testSpec)
      .then((actual) => {
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
      })
  },

  'returns a #x.x.x': async () => {

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

    const testResponse: JsonClientResponse = {
      status: 200,
      data: githubFixtures.tags,
      source: ClientResponseSource.remote
    };

    when(jsonClientMock.get(anything(), anything(), anything()))
      .thenResolve(testResponse)

    // setup initial call
    const cut = new GitHubClient(
      instance(configMock),
      instance(jsonClientMock),
      instance(loggerMock)
    );

    return cut.fetchGithub(testSpec)
      .then((actual) => {
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
      })
  },

  'returns a #sha commit': async () => {

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

    const testResponse: JsonClientResponse = {
      status: 200,
      data: githubFixtures.commits,
      source: ClientResponseSource.remote
    };

    when(jsonClientMock.get(anything(), anything(), anything()))
      .thenResolve(testResponse)

    const cut = new GitHubClient(
      instance(configMock),
      instance(jsonClientMock),
      instance(loggerMock)
    );

    return cut.fetchGithub(testSpec)
      .then((actual) => {
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
      })
  },

  'sets auth token in headers': async () => {

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

    const testToken = 'testToken';

    const testResponse: JsonClientResponse = {
      status: 200,
      data: githubFixtures.commits,
      source: ClientResponseSource.remote
    };

    when(jsonClientMock.get(anything(), anything(), anything()))
      .thenResolve(testResponse)

    when(githubOptsMock.accessToken).thenReturn(testToken);

    const cut = new GitHubClient(
      instance(configMock),
      instance(jsonClientMock),
      instance(loggerMock)
    );

    await cut.fetchGithub(testSpec)

    const [, , actualHeaders] = capture(jsonClientMock.get).first();
    notEqual(actualHeaders, undefined)
    equal(actualHeaders!['authorization'], 'token ' + testToken);
  }

}