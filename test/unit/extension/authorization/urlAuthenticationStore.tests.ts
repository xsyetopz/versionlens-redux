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
        `(Test) ${this.testUrl}`,
        'test label',
        AuthenticationScheme.Basic,
        UrlAuthenticationStatus.NoStatus,
        true
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
      `(Test Update) ${this.testUrl}`,
      'test update label',
      AuthenticationScheme.Basic,
      UrlAuthenticationStatus.NoStatus,
      false
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
    assert.deepEqual(map[this.testUrl], undefined);
  },

  "clears all data": function (this: TestContext) {
    // test
    this.testUrlAuthStore.clear();

    // verify
    verify(this.mockMemento.update('testKey', deepEqual({}))).once();
  },

}