import type { ILogger } from '#domain/logging';
import {
  PackageSourceType,
  SuggestionStatusText,
  VersionUtils,
  PackageDependency,
  createPackageManifest,
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  RubyConfig,
  RubyGitHubClient,
  RubyHttpClient,
  RubySuggestionResolver
} from '#domain/providers/ruby';
import assert, { equal } from 'node:assert';
import { existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
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

  'fromPath: returns a directory suggestion': async function (this: TestContext) {
    const testPackageName = 'test-gem';
    const testPath = './local/gem';
    const testPackageFilePath = resolve('./project/Gemfile');
    const fullContextPath = resolve(dirname(testPackageFilePath), testPath);

    // ensure directory exists
    if (!existsSync(fullContextPath)) mkdirSync(fullContextPath, { recursive: true });

    // test
    const actual = await this.cut.fromPath(testPackageName, testPackageFilePath, testPath);

    // cleanup
    rmSync(dirname(fullContextPath), { recursive: true, force: true });

    // assert
    assert.equal(actual.source, PackageSourceType.Directory);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, fullContextPath);
  },

  'fromGit: returns a git suggestion': function (this: TestContext) {
    // test
    const actual = this.cut.fromGit();

    // assert
    assert.equal(actual.source, PackageSourceType.Git);
  },

  'resolve: returns suggestions from rubygems registry': async function (this: TestContext) {
    const testPackageName = 'rails';
    const testVersion = '6.0.0';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'project/Gemfile'
    );

    const testRequest: any = {
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };
    const testSpec = VersionUtils.parseSemver(testVersion)!;

    when(this.httpClientMock.get(testPackageName))
      .thenResolve({
        data: ['6.0.0', '5.2.0', '5.1.0'],
        source: 'remote' as any,
        status: 200
      });

    // test
    const actual = await this.cut.resolve(testRequest, testSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  },

  'fromGitHub: resolves commits when no ref is provided': async function (this: TestContext) {
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
