import type { ILogger } from '#domain/logging';
import {
  type CargoClient,
  type CargoConfig,
  CargoSuggestionProvider
} from '#domain/providers/cargo';
import { deepEqual } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './cargoSuggestionProvider.fixtures';

type TestContext = {
  cargoClientMock: CargoClient
  cargoConfigMock: CargoConfig
  loggerMock: ILogger
  put: CargoSuggestionProvider
}

export const cargoSuggestionProviderTests = {

  title: CargoSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.cargoClientMock = mock<CargoClient>()
    this.cargoConfigMock = mock<CargoConfig>()
    this.loggerMock = mock<ILogger>()
    this.put = new CargoSuggestionProvider(
      instance(this.cargoClientMock),
      instance(this.cargoConfigMock),
      instance(this.loggerMock)
    )
  },

  "parses dependencies": function (this: TestContext) {
    const testPackagePath = 'test/path/Cargo.yaml'
    const testProps = [
      'package',
      'dependencies',
      'dependencies.*',
      'dev-dependencies',
      'dev-dependencies.*',
      'build-dependencies',
      'build-dependencies.*',
      'workspace.dependencies',
      'workspace.dependencies.*'
    ];
    when(this.cargoConfigMock.dependencyProperties).thenReturn(testProps);
    // test
    const actual = this.put.parseDependencies(testPackagePath, fixtures.test)
    // assert
    deepEqual(actual, fixtures.expected)
  },

}