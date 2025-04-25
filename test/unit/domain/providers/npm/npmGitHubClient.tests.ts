import {
  type GitHubJsonClient,
  type JsonClientResponse,
  ClientResponseSource
} from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { PackageStatusFactory, UpdateableFactory } from '#domain/packages';
import { type NpaSpec, type NpmConfig, NpmGitHubClient } from '#domain/providers/npm';
import { deepEqual, equal } from 'node:assert';
import npa from 'npm-package-arg';
import { anything, instance, mock, when } from 'ts-mockito';
import Fixtures from './npmGitHubClient.fixtures';

type TestContext = {
  configMock: NpmConfig
  githubJsonClientMock: GitHubJsonClient
  loggerMock: ILogger
  cut: NpmGitHubClient
}

export const NpmGitHubClientTests = {

  title: NpmGitHubClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<NpmConfig>();
    this.githubJsonClientMock = mock<GitHubJsonClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new NpmGitHubClient(
      instance(this.configMock),
      instance(this.githubJsonClientMock),
      instance(this.loggerMock)
    );

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
      data: Fixtures.tags,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getTags(anything(), anything()))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchGithub(testSpec)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'range')
    equal(actual.resolved?.name, testRequest.package.name)
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createSatisifiesLatestStatus('v2.5.0'),
        UpdateableFactory.createLatestUpdateable('v2.5.0'),
        UpdateableFactory.createTaggedPreleaseUpdateable('rc', 'v2.6.0-rc.1'),
        UpdateableFactory.createTaggedPreleaseUpdateable('preview', 'v2.5.0-preview.1')
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
      data: Fixtures.tags,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getTags(anything(), anything()))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchGithub(testSpec)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'range')
    equal(actual.resolved?.name, testRequest.package.name)
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createFixedStatus('v2.0.0'),
        UpdateableFactory.createLatestUpdateable('v2.5.0'),
        UpdateableFactory.createTaggedPreleaseUpdateable('rc', 'v2.6.0-rc.1'),
        UpdateableFactory.createTaggedPreleaseUpdateable('preview', 'v2.5.0-preview.1'),
      ]
    )
  },

  'returns a #sha commit': async function (this: TestContext) {

    const testRequest: any = {
      providerName: 'testnpmprovider',
      package: {
        path: 'packagepath',
        name: 'core.js',
        version: 'github:octokit/core.js#166c349',
      }
    };

    const testSpec = npa.resolve(
      testRequest.package.name,
      testRequest.package.version,
      testRequest.package.path
    ) as NpaSpec;

    const testResponse: JsonClientResponse<any> = {
      status: 200,
      data: Fixtures.commits,
      source: ClientResponseSource.remote
    };

    when(this.githubJsonClientMock.getCommits(anything(), anything()))
      .thenResolve(testResponse)

    // test
    const actual = await this.cut.fetchGithub(testSpec)

    // assert
    equal(actual.source, 'github')
    equal(actual.type, 'committish')
    equal(actual.resolved?.name, testRequest.package.name)
    deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createFixedStatus('166c349'),
        UpdateableFactory.createLatestUpdateable('df4d943')
      ]
    )
  },

}