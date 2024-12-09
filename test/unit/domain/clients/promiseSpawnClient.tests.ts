import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { ClientResponseSource, ShellClientRequestError } from '#domain/clients';
import { type PromiseSpawnFn, PromiseSpawnClient } from '#domain/clients/promiseSpawn';
import type { ILogger } from '#domain/logging';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';

interface IPromiseSpawn {
  promiseSpawnFn: PromiseSpawnFn
}

type TestContext = {
  psMock: IPromiseSpawn;
  cachingOptionsMock: CachingOptions;
  loggerMock: ILogger;
}

export const PromiseSpawnClientTests = {

  [test.title]: PromiseSpawnClient.name,

  beforeEach: function (this: TestContext) {
    this.cachingOptionsMock = mock<CachingOptions>()
    this.loggerMock = mock<ILogger>()
    this.psMock = mock<IPromiseSpawn>()
  },

  request: {

    "returns <ShellClientResponse> when error occurs":
      async function (this: TestContext) {
        const testCmd = 'missing';
        const testArgs = ['--ooppss'];
        const testCwd = '/';

        when(this.cachingOptionsMock.duration).thenReturn(30000);

        when(this.psMock.promiseSpawnFn(testCmd, testArgs, anything()))
          .thenReject(<any>{
            code: "ENOENT",
            message: "spawn missing ENOENT"
          });

        const rut = new PromiseSpawnClient(
          instance(this.psMock).promiseSpawnFn,
          new MemoryExpiryCache("PromiseSpawnClientCache"),
          instance(this.cachingOptionsMock),
          instance(this.loggerMock)
        );

        try {
          await rut.request(testCmd, testArgs, testCwd);
        }
        catch (e) {
          const expectedMessage =
            `${ShellClientRequestError.name}:\n`
            + `\tcmd: ${testCmd}\n`
            + `\targs: ${testArgs}\n`
            + `\tcwd: ${testCwd}\n`;

          const error = e as ShellClientRequestError;
          assert.equal(error.message, expectedMessage);
          assert.equal(error.cause.message, "spawn missing ENOENT");
        }

      },

    "returns <ShellClientResponse> and caches response":
      async function (this: TestContext) {
        const testCmd = 'echo';
        const testArgs = ['123'];
        const testCwd = 'd:\\';

        const testResponse = {
          source: ClientResponseSource.cli,
          status: 0,
          data: '123\n',
          rejected: false
        }

        const expectedCacheData = {
          source: ClientResponseSource.cache,
          status: testResponse.status,
          data: testResponse.data,
          rejected: false
        };

        when(this.psMock.promiseSpawnFn(testCmd, testArgs, anything()))
          .thenResolve(<any>{
            code: 0,
            stdout: testResponse.data
          });

        when(this.cachingOptionsMock.duration).thenReturn(30000);

        const rut = new PromiseSpawnClient(
          instance(this.psMock).promiseSpawnFn,
          new MemoryExpiryCache("PromiseSpawnClientCache"),
          instance(this.cachingOptionsMock),
          instance(this.loggerMock)
        );

        const firstResponse = await rut.request(testCmd, testArgs, testCwd);

        assert.deepEqual(firstResponse, testResponse);

        const cachedResponse = await rut.request(testCmd, testArgs, testCwd);

        assert.deepEqual(cachedResponse, expectedCacheData);
      },

    "doesn't cache when duration is 0":
      async function (this: TestContext) {
        const testCmd = 'echo';
        const testArgs = ['123'];
        const testCwd = 'd:\\';

        const testDuration = 0;
        const testKey = 'echo 123';
        const testResponse = {
          source: ClientResponseSource.cli,
          status: 0,
          data: '123\n',
          rejected: false,
        }

        when(this.psMock.promiseSpawnFn(testCmd, testArgs, anything()))
          .thenResolve(<any>{
            code: 0,
            stdout: testResponse.data
          });

        when(this.cachingOptionsMock.duration).thenReturn(testDuration);
        const testCache = new MemoryExpiryCache("PromiseSpawnClientCache");

        const rut = new PromiseSpawnClient(
          instance(this.psMock).promiseSpawnFn,
          testCache,
          instance(this.cachingOptionsMock),
          instance(this.loggerMock)
        );

        const firstResponse = await rut.request(testCmd, testArgs, testCwd);

        assert.deepEqual(firstResponse, testResponse);

        const cachedResponse = await rut.request(testCmd, testArgs, testCwd);

        assert.deepEqual(cachedResponse, testResponse);
        const cachedData = testCache.get(testKey, testDuration);
        assert.equal(cachedData, undefined);
      },

  },

};