import { CachingOptions, ICachingOptions, MemoryExpiryCache } from '#domain/caching';
import { ClientResponseSource, ProcessClientResponse } from '#domain/clients';
import { PromiseSpawnClient } from '#domain/clients/promiseSpawn';
import { ILogger } from '#domain/logging';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';
import { ProcessSpawnStub } from './processSpawnStub';

let psMock: ProcessSpawnStub;
let cachingOptionsMock: ICachingOptions;
let loggerMock: ILogger;

export const ProcessClientRequestTests = {

  [test.title]: PromiseSpawnClient.name,

  beforeEach: () => {
    cachingOptionsMock = mock(CachingOptions)
    loggerMock = mock<ILogger>()
    psMock = mock(ProcessSpawnStub)
  },

  request: {

    "returns <ProcessClientResponse> when error occurs": async () => {

      when(cachingOptionsMock.duration).thenReturn(30000);

      when(psMock.promiseSpawn(anything(), anything(), anything()))
        .thenReject(<any>{
          code: "ENOENT",
          message: "spawn missing ENOENT"
        });

      const rut = new PromiseSpawnClient(
        instance(psMock).promiseSpawn,
        new MemoryExpiryCache("PromiseSpawnClientCache"),
        instance(cachingOptionsMock),
        instance(loggerMock)
      );

      try {
        await rut.request(
          'missing',
          ['--ooppss'],
          '/'
        );
      }
      catch (error) {
        const response = error as ProcessClientResponse;
        assert.equal(response.status, "ENOENT");
        assert.equal(response.data, "spawn missing ENOENT");
        assert.equal(response.rejected, true);
      }

    },

    "returns <ProcessClientResponse> and caches response": async () => {
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

      when(psMock.promiseSpawn(anything(), anything(), anything()))
        .thenResolve(<any>{
          code: 0,
          stdout: testResponse.data
        });

      when(cachingOptionsMock.duration).thenReturn(30000);

      const rut = new PromiseSpawnClient(
        instance(psMock).promiseSpawn,
        new MemoryExpiryCache("PromiseSpawnClientCache"),
        instance(cachingOptionsMock),
        instance(loggerMock)
      );

      const firstResponse = await rut.request(
        'echo',
        ['123'],
        'd:\\'
      );

      assert.deepEqual(firstResponse, testResponse);

      const cachedResponse = await rut.request(
        'echo',
        ['123'],
        'd:\\'
      );

      assert.deepEqual(cachedResponse, expectedCacheData);
    },

    "doesn't cache when duration is 0": async () => {
      const testDuration = 0;
      const testKey = 'echo 123';
      const testResponse = {
        source: ClientResponseSource.cli,
        status: 0,
        data: '123\n',
        rejected: false,
      }

      when(psMock.promiseSpawn(anything(), anything(), anything()))
        .thenResolve(<any>{
          code: 0,
          stdout: testResponse.data
        });

      when(cachingOptionsMock.duration).thenReturn(testDuration);
      const testCache = new MemoryExpiryCache("PromiseSpawnClientCache");

      const rut = new PromiseSpawnClient(
        instance(psMock).promiseSpawn,
        testCache,
        instance(cachingOptionsMock),
        instance(loggerMock)
      );

      const firstResponse = await rut.request(
        'echo',
        ['123'],
        'd:\\'
      );

      assert.deepEqual(firstResponse, testResponse);

      const cachedResponse = await rut.request(
        'echo',
        ['123'],
        'd:\\'
      );

      assert.deepEqual(cachedResponse, testResponse);
      const cachedData = testCache.get(testKey, testDuration);
      assert.equal(cachedData, undefined);
    },

  },

};