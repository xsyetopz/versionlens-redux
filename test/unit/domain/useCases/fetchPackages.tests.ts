import type { ILogger } from '#domain/logging';
import {
  type PackageResponse,
  type TPackageClientRequest,
  createPackageResource,
  PackageCache,
  PackageDependency,
  PackageVersionType
} from '#domain/packages';
import { createTextRange, PackageDescriptor } from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import { FetchPackage, FetchPackages } from '#domain/useCases';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { anything, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  fetchPackageSuggestionsMock: FetchPackage,
  loggerMock: ILogger,
  suggestionProviderMock: ISuggestionProvider,
  testPackageCache: PackageCache
}

export const fetchPackagesTests = {

  [test.title]: FetchPackages.name,

  beforeEach: function (this: TestContext) {
    this.loggerMock = mock<ILogger>();
    this.fetchPackageSuggestionsMock = mock<FetchPackage>();

    this.suggestionProviderMock = mock<ISuggestionProvider>();
    when(this.suggestionProviderMock.name).thenReturn("test provider");

    this.testPackageCache = new PackageCache([this.suggestionProviderMock.name]);
  },

  "fetches suggestions $1": [
    ['without prefetch', false],
    ['with prefetch', true],
    async function (this: TestContext, testCaseTitle: string, testPreFetch: boolean) {
      const testProjectPath = 'test/project/path';
      const testPackagePath = 'test/package/path';

      const usecase = new FetchPackages(
        instance(this.fetchPackageSuggestionsMock),
        instance(this.loggerMock)
      );

      const testDependencies = [
        new PackageDependency(
          createPackageResource(
            "testPackage1",
            "1.0.0",
            "test/path"
          ),
          //nameRange
          createTextRange(0, 1),
          // versionRange
          createTextRange(2, 3),
          new PackageDescriptor([])
        )
      ];

      const testProvider = instance(this.suggestionProviderMock);

      const testClientData = 'test client data';
      if (testPreFetch) {
        when(this.suggestionProviderMock.preFetchSuggestions!(testProjectPath, testPackagePath))
          .thenResolve(testClientData);
      }

      when(this.fetchPackageSuggestionsMock.execute(anything(), anything()))
        .thenCall((prov: ISuggestionProvider, cr: TPackageClientRequest<any>) => {
          const testResp = <PackageResponse>{
            order: 0,
            type: PackageVersionType.Version,
            providerName: prov.name,
            parsedDependency: cr.parsedDependency,
          };

          return testResp;
        });

      // test
      const actual = await usecase.execute(
        testProvider,
        testProjectPath,
        testPackagePath,
        testDependencies
      );

      // assert
      assert.ok(actual.length > 0);
      actual.forEach(
        (result, index) => {
          assert.equal(result.order, 0);
          assert.equal(result.type, PackageVersionType.Version);
          assert.equal(result.providerName, testProvider.name);
          assert.equal(result.parsedDependency, testDependencies[index]);
        }
      );

      if (testPreFetch) {
        verify(
          this.suggestionProviderMock.preFetchSuggestions!(anything(), anything())
        ).once();
      }

      verify(
        this.loggerMock.debug(
          "queueing {packageCount} package fetch tasks",
          testDependencies.length
        )
      ).once();

      verify(
        this.loggerMock.info(
          "all packages fetched for {providerName} ({duration} ms)",
          testProvider.name,
          anything()
        )
      ).once();

    }
  ],

};