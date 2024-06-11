import { TTomlPackageParserOptions, parsePackagesToml } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import Fixtures from './extractPackageDependenciesFromToml.fixtures';

export const extractPackageDependenciesFromTomlTests = {

  [test.title]: parsePackagesToml.name,

  "returns empty when no dependency entry names match": () => {
    const includePropNames = ["non-dependencies"];

    const testOptions: TTomlPackageParserOptions = {
      includePropNames
    };

    const results = parsePackagesToml(
      Fixtures.parsesDependencyEntries.test,
      testOptions
    );

    assert.equal(results.length, 0);
  },

  "case $i: parses dependencies from toml": [
    Fixtures.parsesDependencyEntries,
    Fixtures.parsesProjectVersionEntries,
    (fixture: any) => {
      const testOptions = {
        includePropNames: [
          'project',
          'package',
          'dependencies',
          'dependencies.*',
          'dev-dependencies',
          'tool.poetry.group.*.dependencies',
        ]
      }
      const actual = parsePackagesToml(fixture.test, testOptions)
      assert.deepEqual(actual, fixture.expected);
    }
  ]

}