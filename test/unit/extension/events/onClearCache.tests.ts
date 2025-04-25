import type { IExpiryCache } from '#domain/caching';
import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import { OnClearCache } from '#extension/events';
import { test } from 'mocha-ui-esm';
import { instance, mock, verify } from 'ts-mockito';

type TestContext = {
  mockPackageCache: PackageCache
  mockShellCache: IExpiryCache
  mockUrlRequestCache: IExpiryCache
  mockLogger: ILogger
}

export const onClearCacheTests = {

  [test.title]: OnClearCache.name,

  beforeEach: function (this: TestContext) {
    this.mockPackageCache = mock<PackageCache>();
    this.mockShellCache = mock<IExpiryCache>();
    this.mockUrlRequestCache = mock<IExpiryCache>();
    this.mockLogger = mock<ILogger>();
  },

  "clears package caches": function (this: TestContext) {
    const testEvent = new OnClearCache(
      instance(this.mockPackageCache),
      instance(this.mockShellCache),
      instance(this.mockUrlRequestCache),
      instance(this.mockLogger)
    );

    // test
    testEvent.execute();

    // verify
    verify(this.mockLogger.debug("Clearing package caches")).once();
    verify(this.mockPackageCache.clear()).once();
    verify(this.mockShellCache.clear()).once();
    verify(this.mockUrlRequestCache.clear()).once();
  },

};