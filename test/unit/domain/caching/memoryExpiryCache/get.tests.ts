import assert from "assert";
import { MemoryExpiryCache } from "domain/caching";
import { test } from "mocha-ui-esm";

let testCacheMap: MemoryExpiryCache;

export const getTests = {

  [test.title]: MemoryExpiryCache.prototype.get.name,

  beforeEach: () => {
    testCacheMap = new MemoryExpiryCache("testMemoryExpiryCache")
  },

  "returns undefined if the key does not exist": () => {
    // setup
    const testKey = 'missing';

    // test
    const actual = testCacheMap.get(testKey, 1000);

    // assert
    assert.equal(actual, undefined);
  },

  "returns entry when unexpired": () => {
    // setup
    const testKey = 'key1';
    const testData = { "test": 123 };
    testCacheMap.set(testKey, testData);

    // test
    const actual = testCacheMap.get(testKey, 1000);

    // assert
    assert.deepEqual(actual, testData);
  },

  "returns undefined when expired": () => {
    // setup
    const testKey = 'key1';
    const testData = { "test": 123 };
    testCacheMap.set(testKey, testData);

    // test
    const actual = testCacheMap.get(testKey, 0);

    // assert
    assert.equal(actual, undefined);
  }

}