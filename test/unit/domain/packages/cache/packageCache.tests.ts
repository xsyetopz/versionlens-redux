import {
  type PackageClientResponse,
  type PackageResource,
  PackageCache,
  PackageSourceType,
  PackageVersionType
} from '#domain/packages';
import { test } from "mocha-ui-esm";
import { deepEqual, fail } from 'node:assert';

type TestContext = {
  testProviderName: string;
  testPackageFilePath: string;
  testPackage: PackageResource;
  testCacheItem: PackageClientResponse;
  testCache: PackageCache;
}

export const packageCacheTests = {

  [test.title]: PackageCache.name,

  beforeEach: function (this: TestContext) {
    this.testProviderName = "test";
    this.testPackage = {
      name: "test-package",
      version: "1.2.3",
      path: ""
    };
    this.testCacheItem = {
      source: PackageSourceType.Registry,
      type: PackageVersionType.Version,
      suggestions: []
    };
    this.testCache = new PackageCache([this.testProviderName]);
  },

  create: async function (this: TestContext) {
    // test
    const actual = await this.testCache.getOrCreate(
      this.testProviderName,
      this.testPackage,
      async () => this.testCacheItem,
      1000
    );

    // assert
    deepEqual(actual, this.testCacheItem);
  },

  get: async function (this: TestContext) {
    // setup
    await this.testCache.getOrCreate(
      this.testProviderName,
      this.testPackage,
      async () => this.testCacheItem,
      1000
    );

    // test
    const actual = await this.testCache.getOrCreate(
      this.testProviderName,
      this.testPackage,
      async () => fail('should not call method to cache'),
      1000
    );

    // assert
    deepEqual(actual, this.testCacheItem);
  },

  clear: async function (this: TestContext) {
    // setup
    const expectedData = this.testCacheItem = {
      source: PackageSourceType.File,
      type: PackageVersionType.Tag,
      suggestions: []
    };

    await this.testCache.getOrCreate(
      this.testProviderName,
      this.testPackage,
      async () => this.testCacheItem,
      1000
    );

    // test
    this.testCache.clear();

    // assert
    const actual = await this.testCache.getOrCreate(
      this.testProviderName,
      this.testPackage,
      async () => expectedData,
      1000
    );

    deepEqual(actual, expectedData);
  },

}