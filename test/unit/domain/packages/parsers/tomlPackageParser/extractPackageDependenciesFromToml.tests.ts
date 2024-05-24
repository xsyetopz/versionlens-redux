import assert from 'node:assert';
import { TTomlPackageParserOptions, parsePackagesToml } from 'domain/packages';
import { test } from 'mocha-ui-esm';
import Fixtures from './extractPackageDependenciesFromToml.fixtures';

export const extractPackageDependenciesFromTomlTests = {

  [test.title]: parsePackagesToml.name,

  "returns empty when no dependency entry names match": () => {
    const includePropNames = ["non-dependencies"];

    const testOptions: TTomlPackageParserOptions = {
      includePropNames
    };

    const results = parsePackagesToml(
      Fixtures.extractDependencyEntries.test,
      testOptions
    );

    assert.equal(results.length, 0);
  },

  "extracts dependencies from toml": () => {
    const testOptions = {
      includePropNames: [
        'dependencies',
        'dependencies.*',
        'dev-dependencies',
        'tool.poetry.group.*.dependencies',
      ]
    }

    const actual = parsePackagesToml(Fixtures.extractDependencyEntries.test, testOptions)

    assert.deepEqual(actual, Fixtures.extractDependencyEntries.expected);
  }

}