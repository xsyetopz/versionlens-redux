import assert from 'node:assert';
import {
  createPathDescFromJsonNode,
  createRepoDescFromJsonNode,
  createVersionDescFromJsonNode,
  parsePackagesJson,
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler
} from 'domain/packages';
import { KeyDictionary } from 'domain/utils';
import { test } from 'mocha-ui-esm';
import Fixtures from './extractPackageDependenciesFromJson.fixtures';

const complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler> = {
  "version": createVersionDescFromJsonNode,
  "path": createPathDescFromJsonNode,
  "repository": createRepoDescFromJsonNode
};

export const extractPackageDependenciesFromJsonTests = {

  [test.title]: parsePackagesJson.name,

  "returns empty when no matches found": () => {
    const includePropNames: Array<string> = []

    const testOptions: TJsonPackageParserOptions = {
      includePropNames,
      complexTypeHandlers
    };

    const results = parsePackagesJson(
      "",
      testOptions
    );
    assert.equal(results.length, 0);
  },

  "returns empty when no dependency entry names match": () => {
    const includePropNames = ["non-dependencies"];

    const testOptions: TJsonPackageParserOptions = {
      includePropNames,
      complexTypeHandlers
    };

    const results = parsePackagesJson(
      JSON.stringify(Fixtures.extractDependencyEntries.test),
      testOptions
    );

    assert.equal(results.length, 0);
  },

  "extracts dependency entries from json": () => {
    const includePropNames = ["dependencies"];

    const testOptions: TJsonPackageParserOptions = {
      includePropNames,
      complexTypeHandlers
    };

    const results = parsePackagesJson(
      JSON.stringify(Fixtures.extractDependencyEntries.test),
      testOptions
    );

    assert.deepEqual(results, Fixtures.extractDependencyEntries.expected);
  },

  "matches json expression paths": () => {
    const includePropNames = ["overrides.*"];

    const testOptions: TJsonPackageParserOptions = {
      includePropNames,
      complexTypeHandlers
    };

    const results = parsePackagesJson(
      JSON.stringify(Fixtures.matchesPathExpressions.test),
      testOptions
    );

    assert.deepEqual(results, Fixtures.matchesPathExpressions.expected);
  },

  "matches json packageManager": () => {
    const includePropNames = ["packageManager"];

    const testOptions: TJsonPackageParserOptions = {
      includePropNames,
      complexTypeHandlers,
    };

    const results = parsePackagesJson(
      JSON.stringify(Fixtures.matchesPackageManagerExpressions.test),
      testOptions
    );

    assert.deepEqual(
      results,
      Fixtures.matchesPackageManagerExpressions.expected
    );
  },
};
