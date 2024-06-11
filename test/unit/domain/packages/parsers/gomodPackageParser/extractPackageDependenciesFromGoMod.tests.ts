import { parsePackagesGoMod } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import Fixtures from './extractPackageDependenciesFromGoMod.fixtures';

export const extractPackageDependenciesFromGoModTests = {

  [test.title]: parsePackagesGoMod.name,

  "returns empty when no dependency entry names match": () => {
    const results = parsePackagesGoMod(Fixtures.extractNoDependencies.test);

    assert.equal(results.length, 0);
  },

  "extracts dependencies from toml": () => {
    const actual = parsePackagesGoMod(Fixtures.extractDependencies.test);

    assert.deepEqual(actual, Fixtures.extractDependencies.expected);
  }

}