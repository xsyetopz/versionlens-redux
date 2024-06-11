import assert from 'node:assert';
import { Config, IConfig } from '#domain/configuration';
import { test } from 'mocha-ui-esm';
import { instance, mock, when } from 'ts-mockito';
import { ConfigResolverStub } from './stubs/configResolverStub';

let configResolverMock: ConfigResolverStub;

export const AppConfigTests = {

  [test.title]: Config.name,

  beforeAll: () => {
    configResolverMock = mock(ConfigResolverStub);
  },

  get: {

    "accesses frozen repo after first call": () => {
      const testSection = 'testsection'
      const testKey = 'some_property';
      let expectedFrozenValue = 'test value';

      when(configResolverMock.getConfiguration(testSection))
        .thenReturn(<IConfig>{
          get: (section) => expectedFrozenValue
        })

      // get original value
      const cut = new Config(instance(configResolverMock).getConfiguration, testSection);
      const first = cut.get(testKey);
      assert.equal(first, expectedFrozenValue)

      // change value without defrosting
      when(configResolverMock.getConfiguration(testSection))
        .thenReturn(
          <IConfig>{
            get: (section) => 'hot value'
          }
        )

      // should still return original value
      const actualFrozen = cut.get(testKey);

      assert.equal(actualFrozen, expectedFrozenValue)
    },

    "returns hot value after defrost is called": () => {
      const testSection = 'testsection'
      const testKey = 'some_property';
      let initialValue = 'test value';

      when(configResolverMock.getConfiguration(testSection))
        .thenReturn(
          <IConfig>{
            get: (section) => initialValue
          }
        )

      // get original value
      const cut = new Config(instance(configResolverMock).getConfiguration, testSection);
      const first = cut.get(testKey);
      assert.equal(first, initialValue)

      // change the value
      const expectedHotValue = 'hot value';
      when(configResolverMock.getConfiguration(testSection))
        .thenReturn(
          <IConfig>{
            get: (section) => expectedHotValue
          }
        )

      // should return the new value after defrost is called
      cut.defrost();
      const actualFrozen = cut.get(testKey);

      assert.equal(actualFrozen, expectedHotValue)
    }

  }

}