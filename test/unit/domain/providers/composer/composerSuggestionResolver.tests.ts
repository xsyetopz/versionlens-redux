import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  createPackageManifest,
  PackageDependency,
  PackageSourceType,
  VersionUtils
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  type ComposerConfig,
  type PackagistClient,
  ComposerSuggestionResolver
} from '#domain/providers/composer';
import assert, { deepEqual } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './composerSuggestionResolver.fixtures';

type TestContext = {
  configMock: ComposerConfig
  packagistClientMock: PackagistClient
  loggerMock: ILogger
  cut: ComposerSuggestionResolver
}

export const ComposerSuggestionResolverTests = {

  title: ComposerSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<ComposerConfig>();
    this.packagistClientMock = mock<PackagistClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new ComposerSuggestionResolver(
      instance(this.configMock),
      instance(this.packagistClientMock),
      instance(this.loggerMock)
    );
  },

  fromPackagist: {
    'case $i: returns suggestions': [
      ['v3.1.3', Fixtures.registryVersion.expected1],
      ['v3.0', Fixtures.registryVersion.expected2],
      async function (this: TestContext, testVersion: string, expected: any) {
        const testPackageName = 'php-parallel-lint/php-parallel-lint'
        const testPackageMan = createPackageManifest(
          // package name
          testPackageName,
          // package version
          testVersion,
          // package path
          'packagepath',
        );
        const testSpec = VersionUtils.parseSemver(testPackageMan.version)!;
        const testRequest: PackageClientRequest<null> = {
          providerName: 'test-composer-provider',
          clientData: null,
          parsedDependency: new PackageDependency(
            testPackageMan,
            new PackageDescriptor([
              createPackageNameDesc(testPackageMan.name, createTextRange(0)),
              createPackageVersionDesc(testPackageMan.version, createTextRange(1)),
            ]),
          )
        }

        when(this.packagistClientMock.get(testPackageName))
          .thenResolve({
            data: Fixtures.registryVersion.test,
            source: ClientResponseSource.remote,
            status: 200
          })

        // test
        const actual = await this.cut.fromPackagist(testRequest, testSpec)

        // assert
        deepEqual(actual.suggestions, expected)
      }
    ],

    'fromPackagist: returns suggestions for version ranges': async function (this: TestContext) {
      const testPackageName = 'test-package';
      const testVersion = '^1.0.0';
      const testPackageMan = createPackageManifest(
        testPackageName,
        testVersion,
        'packagepath'
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

      const testSpec: any = {
        type: 'range',
        rawVersion: testVersion
      }

      const testReleases = ['1.0.0', '1.1.0', '1.2.0'];
      const testResponse: any = {
        source: ClientResponseSource.remote,
        status: 200,
        data: {
          packages: {
            [testPackageName]: testReleases.map(version => ({ version }))
          }
        }
      };
      when(this.packagistClientMock.get(testPackageName))
        .thenResolve(testResponse);

      // test
      const actual = await this.cut.fromPackagist(testRequest, testSpec)

      // assert
      assert.equal(actual.source, PackageSourceType.Registry)
      assert.equal(actual.type, testSpec.type)
      assert.ok(actual.suggestions.length > 0)
    }
  }
}