import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  createPackageResource,
  PackageDependency,
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
import { deepEqual } from 'node:assert';
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
        const testPackageRes = createPackageResource(
          // package name
          testPackageName,
          // package version
          testVersion,
          // package path
          'packagepath',
        );
        const testSpec = VersionUtils.parseSemver(testPackageRes.version)!;
        const testRequest: PackageClientRequest<null> = {
          providerName: 'test-composer-provider',
          clientData: null,
          parsedDependency: new PackageDependency(
            testPackageRes,
            new PackageDescriptor([
              createPackageNameDesc(testPackageRes.name, createTextRange(0)),
              createPackageVersionDesc(testPackageRes.version, createTextRange(1)),
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
      },
    ]
  }
}