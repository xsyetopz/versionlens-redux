import type { KeyDictionary } from '#domain/utils';
import {
  type UrlAuthenticationData,
  AuthenticationScheme,
  createUrlAuthData,
  UrlAuthenticationStatus,
  UrlAuthenticationStore
} from '#extension/authorization';
import assert from 'assert';
import { test } from 'mocha-ui-esm';
import { anyOfClass, deepEqual, instance, mock, verify, when } from 'ts-mockito';
import type { Memento } from 'vscode';

type TestContext = {
  mockMemento: Memento
  testStoreKey: 'testKey'
  testUrlAuthStore: UrlAuthenticationStore
  testUrl: string
  testData: KeyDictionary<UrlAuthenticationData>
}

export const UrlAuthenticationStoreTests = {

  [test.title]: UrlAuthenticationStore.name,

  beforeEach: function (this: TestContext) {
    this.mockMemento = mock<Memento>();
    this.testStoreKey = 'testKey';
    this.testUrlAuthStore = new UrlAuthenticationStore(this.testStoreKey, instance(this.mockMemento));
    this.testUrl = 'http://anything';
    this.testData = {
      [this.testUrl]: createUrlAuthData(
        this.testUrl,
        AuthenticationScheme.Basic,
        'test label',
        UrlAuthenticationStatus.NoStatus
      )
    };
    when(this.mockMemento.get(this.testStoreKey, deepEqual({}))).thenReturn(this.testData);
  },

  "gets data by url": function (this: TestContext) {
    // test
    const actual = this.testUrlAuthStore.get(this.testUrl);

    // verify
    verify(this.mockMemento.get(this.testStoreKey, deepEqual({}))).once();

    // assert
    assert.deepEqual(actual, this.testData[this.testUrl]);
  },

  "gets all data by url": function (this: TestContext) {
    // test
    const actual = this.testUrlAuthStore.getAll();

    // verify
    verify(this.mockMemento.get(this.testStoreKey, deepEqual({}))).once();

    // assert
    assert.equal(actual.length, 1);
    assert.deepEqual(actual[0], this.testData[this.testUrl]);
  },

  "updates data by url": async function (this: TestContext) {
    const testUpdateData = createUrlAuthData(
      this.testUrl,
      AuthenticationScheme.Basic,
      'test update label',
      UrlAuthenticationStatus.NoStatus
    );

    // test
    await this.testUrlAuthStore.update(this.testUrl, testUpdateData);

    // verify
    verify(this.mockMemento.get(this.testStoreKey, deepEqual({}))).once();
    verify(this.mockMemento.update(this.testStoreKey, anyOfClass(Object))).once();

    // assert
    const actual = this.testUrlAuthStore.get(this.testUrl);
    assert.deepEqual(actual, testUpdateData);
  },

  "removes data by url": function (this: TestContext) {
    // test
    this.testUrlAuthStore.remove(this.testUrl);

    // verify
    verify(this.mockMemento.get(this.testStoreKey, deepEqual({}))).once();
    verify(this.mockMemento.update('testKey', deepEqual({}))).once();

    // assert
    const map = this.testUrlAuthStore.getAll();
    assert.equal(map.length, 0)
  },

  "clears all data": function (this: TestContext) {
    // test
    this.testUrlAuthStore.clear();

    // verify
    verify(this.mockMemento.update('testKey', deepEqual({}))).once();
  },

}