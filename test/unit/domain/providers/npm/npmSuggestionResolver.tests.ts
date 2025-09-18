import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageSuggestion,
  createPackageResource,
  PackageDependency,
  PackageSourceType,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  type NpaSpec,
  type NpmClientData,
  type NpmConfig,
  type NpmGitHubClient,
  type NpmRegistryClient,
  NpmSuggestionResolver,
} from '#domain/providers/npm';
import { fileDir } from '#test/utils';
import assert, { deepEqual, fail } from 'node:assert';
import npa from 'npm-package-arg';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './npmSuggestionResolver.fixtures';

const testDir = fileDir();

type TestContext = {
  configMock: NpmConfig
  npmRegistryClientMock: NpmRegistryClient
  githubClientMock: NpmGitHubClient
  loggerMock: ILogger
  cut: NpmSuggestionResolver
}

export const NpmSuggestionResolverTests = {

  title: NpmSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<NpmConfig>();
    this.npmRegistryClientMock = mock<NpmRegistryClient>();
    this.githubClientMock = mock<NpmGitHubClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new NpmSuggestionResolver(
      instance(this.configMock),
      instance(this.npmRegistryClientMock),
      instance(this.githubClientMock),
      instance(this.loggerMock)
    );
  },

  'returns a file:// directory package': async function (this: TestContext) {
    const expectedSource = 'directory';
    const testPackageRes = createPackageResource(
      // package name
      'filepackage',
      // package version
      'file:../../..',
      // package path
      testDir,
    );

    // test
    const actual = await this.cut.fromFileProtocol(testPackageRes)

    // assert
    assert.equal(actual.source, PackageSourceType.Directory, `expected to see ${expectedSource}`)
    assert.deepEqual(actual.resolved?.name, testPackageRes.name)
  },

  fromGit: {
    'returns fixed package for git:// requests': async function (this: TestContext) {
      const testPackageRes = createPackageResource(
        // package name
        'core.js',
        // package version
        'git+https://git@github.com/testuser/test.git',
        // package path
        'packagepath',
      );
      const testNpaSpec = npa.resolve(
        testPackageRes.name,
        testPackageRes.version,
        testPackageRes.path
      ) as NpaSpec;

      // test
      const actual = await this.cut.fromGit(testNpaSpec)

      // assert
      assert.equal(actual.source, 'git')
      assert.equal(actual.resolved, null)
      assert.deepEqual(
        actual.suggestions,
        [
          <PackageSuggestion>{
            name: SuggestionStatusText.Fixed,
            category: SuggestionCategory.Match,
            version: 'git repository',
            type: SuggestionTypes.status
          }
        ]
      )
    },
    'returns unsupported suggestion when not github': async function (this: TestContext) {
      const testPackageRes = createPackageResource(
        // package name
        'core.js',
        // package version
        'git+https://git@not-gihub.com/testuser/test.git',
        // package path
        'packagepath',
      );
      const testNpaSpec = npa.resolve(
        testPackageRes.name,
        testPackageRes.version,
        testPackageRes.path
      ) as NpaSpec;
      // test
      try {
        await this.cut.fromGit(testNpaSpec)
        fail()
      } catch (error) {
        const actual = error as PackageClientResponse
        deepEqual(
          actual,
          {
            status: 'EUNSUPPORTEDPROTOCOL',
            data: 'Git url could not be resolved',
            source: ClientResponseSource.local
          }
        )
      }
    },
  },

  fromRegistry: {
    'returns a registry alias package': async function (this: TestContext) {
      const testPackageRes = createPackageResource(
        // package name
        'aliased',
        // package version
        'npm:pacote@11.1.9',
        // package path
        'packagepath',
      );
      const testClientData: NpmClientData = {
        registry: 'https://registry.npmjs.org/',
        strictSSL: true
      };
      const testRequest: PackageClientRequest<NpmClientData> = {
        providerName: 'testnpmprovider',
        clientData: testClientData,
        parsedDependency: new PackageDependency(
          testPackageRes,
          new PackageDescriptor([
            createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
            createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
          ]),
        )
      }
      const testNpaSpec = npa.resolve(
        testPackageRes.name,
        testPackageRes.version,
        testPackageRes.path
      ) as NpaSpec;

      when(this.npmRegistryClientMock.get(testNpaSpec.subSpec, testClientData))
        .thenResolve(
          {
            data: Fixtures.packumentRegistryAlias,
            source: ClientResponseSource.remote,
            status: 200
          }
        )

      // test
      const actual = await this.cut.fromRegistry(testRequest, testNpaSpec)

      // assert
      assert.equal(actual.source, 'registry')
      assert.equal(actual.type, 'alias')
      assert.equal(actual.resolved?.name, 'pacote')
      assert.equal(actual.resolved?.version, '11.1.9')
    },
    'case $i: returns capped latest versions': [
      ['7.0.0', Fixtures.cappedToLatestTaggedRelease],
      ['*', Fixtures.cappedToLatestTaggedRelease],
      async function (this: TestContext, testVersion: string, fixture: any) {
        const testPackageRes = createPackageResource(
          // package name
          'npm-package-arg',
          // package version
          testVersion,
          // package path
          'packagepath',
        );
        const testClientData: NpmClientData = {
          registry: 'https://registry.npmjs.org/',
          strictSSL: true
        };
        const testRequest: PackageClientRequest<NpmClientData> = {
          providerName: 'testnpmprovider',
          clientData: testClientData,
          parsedDependency: new PackageDependency(
            testPackageRes,
            new PackageDescriptor([
              createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
              createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
            ]),
          )
        }
        const testNpaSpec = npa.resolve(
          testPackageRes.name,
          testPackageRes.version,
          testPackageRes.path
        ) as NpaSpec;

        when(this.npmRegistryClientMock.get(testNpaSpec, testClientData))
          .thenResolve(
            {
              data: fixture.test,
              source: ClientResponseSource.remote,
              status: 200
            }
          )

        // test
        const actual = await this.cut.fromRegistry(testRequest, testNpaSpec)
        // assert
        assert.deepEqual(actual.suggestions, fixture.expected)
      }
    ],
    'returns a registry version package': async function (this: TestContext) {
      const testPackageRes = createPackageResource(
        // package name
        'npm-package-arg',
        // package version
        '8.0.1',
        // package path
        'packagepath',
      );

      const testClientData: NpmClientData = {
        registry: 'https://registry.npmjs.org/',
        strictSSL: true
      };

      const testRequest: PackageClientRequest<NpmClientData> = {
        providerName: 'testnpmprovider',
        clientData: testClientData,
        parsedDependency: new PackageDependency(
          testPackageRes,
          new PackageDescriptor([
            createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
            createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
          ]),
        )
      }

      const testNpaSpec = npa.resolve(
        testPackageRes.name,
        testPackageRes.version,
        testPackageRes.path
      ) as NpaSpec;

      when(this.npmRegistryClientMock.get(testNpaSpec, testClientData))
        .thenResolve(
          {
            data: Fixtures.packumentRegistryVersion,
            source: ClientResponseSource.remote,
            status: 200
          }
        )

      // test
      const actual = await this.cut.fromRegistry(testRequest, testNpaSpec)

      // assert
      assert.equal(actual.source, 'registry')
      assert.equal(actual.type, 'version')
      assert.equal(actual.resolved?.name, testPackageRes.name)
    },
    'returns a registry range package': async function (this: TestContext) {
      const testPackageRes = createPackageResource(
        // package name
        'pacote',
        // package version
        '10.1.*',
        // package path
        'packagepath',
      );

      const testClientData: NpmClientData = {
        registry: 'https://registry.npmjs.org/',
        strictSSL: true
      };

      const testRequest: PackageClientRequest<NpmClientData> = {
        providerName: 'testnpmprovider',
        clientData: testClientData,
        parsedDependency: new PackageDependency(
          testPackageRes,
          new PackageDescriptor([
            createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
            createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
          ]),
        )
      }

      const testNpaSpec = npa.resolve(
        testPackageRes.name,
        testPackageRes.version,
        testPackageRes.path
      ) as NpaSpec;

      when(this.npmRegistryClientMock.get(testNpaSpec, testClientData))
        .thenResolve(
          {
            data: Fixtures.packumentRegistryRange,
            source: ClientResponseSource.remote,
            status: 200
          }
        )

      // test
      const actual = await this.cut.fromRegistry(testRequest, testNpaSpec)

      // assert
      assert.equal(actual.source, 'registry')
      assert.equal(actual.type, 'range')
      assert.equal(actual.resolved?.name, testPackageRes.name)
      assert.deepEqual(actual.resolved?.version, testPackageRes.version)
    },
  }
}