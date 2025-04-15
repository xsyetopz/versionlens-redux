import assert from "node:assert";
import { DependencyCache, PackageDependency } from '#domain/packages';
import { test } from "mocha-ui-esm";
import { anything, instance, mock, verify, when } from "ts-mockito";

type TestContext = {
  testProviderName: string;
  testPackageFilePath: string;
}

export const dependencyCacheTests = {

  [test.title]: DependencyCache.name,

  beforeEach: function (this: TestContext) {
    this.testProviderName = "test";
    this.testPackageFilePath = "some/path/package.json";
  },

  get: function (this: TestContext) {
    // setup
    const testDeps: PackageDependency[] = [<any>{ package: { name: "test", version: "1.2.3" } }];
    const cache = new DependencyCache([this.testProviderName]);

    cache.set(this.testProviderName, this.testPackageFilePath, testDeps)

    // test
    const actual = cache.get(this.testProviderName, this.testPackageFilePath);

    // assert
    assert.deepEqual(actual, testDeps);
  },

  remove: function (this: TestContext) {
    // setup
    const testDeps: PackageDependency[] = [];
    const cache = new DependencyCache([this.testProviderName]);

    cache.set(this.testProviderName, this.testPackageFilePath, testDeps)

    // test
    cache.remove(this.testProviderName, this.testPackageFilePath);

    // assert
    const removed = cache.get(this.testProviderName, this.testPackageFilePath);
    assert.equal(removed, undefined);
  },

  getDependenciesWithFallback: {

    beforeEach: function (this: TestContext) {
      this.testProviderName = "test";
      this.testPackageFilePath = "some/path/package.json";
    },

    "uses preferred cache to fetch dependencies": function (this: TestContext) {
      const testPreferredDeps: PackageDependency[] = [];

      const mockPreferredCache = mock<DependencyCache>();
      const mockFallbackCache = mock<DependencyCache>();

      when(mockPreferredCache.get(anything(), anything()))
        .thenReturn(testPreferredDeps);

      // test
      const actualDeps = DependencyCache.getDependenciesWithFallback(
        this.testProviderName,
        this.testPackageFilePath,
        instance(mockPreferredCache),
        instance(mockFallbackCache)
      );

      // verify 
      verify(mockPreferredCache.get(this.testProviderName, this.testPackageFilePath))
        .once();

      verify(mockFallbackCache.get(this.testProviderName, this.testPackageFilePath))
        .never();

      // assert
      assert.equal(actualDeps, testPreferredDeps);
    },

    "uses fallback cache to fetch dependencies": function (this: TestContext) {
      const testFallbackDeps: PackageDependency[] = [];
      const mockPreferredCache = mock<DependencyCache>();
      const mockFallbackCache = mock<DependencyCache>();

      when(mockFallbackCache.get(this.testProviderName, this.testPackageFilePath))
        .thenReturn(testFallbackDeps);

      // test
      const actualDeps = DependencyCache.getDependenciesWithFallback(
        this.testProviderName,
        this.testPackageFilePath,
        instance(mockPreferredCache),
        instance(mockFallbackCache)
      );

      // verify 
      verify(mockPreferredCache.get(this.testProviderName, this.testPackageFilePath))
        .once();

      verify(mockFallbackCache.get(this.testProviderName, this.testPackageFilePath))
        .once();

      // assert
      assert.equal(actualDeps, testFallbackDeps);
    }

  }

}