import {
  type JsonPackageTypeHandler,
  type JsonParserOptions,
  createPathDescFromJsonNode,
  createRepoDescFromJsonNode,
  createVersionDescFromJsonNode,
  parsePackagesJson
} from '#domain/parsers';
import { customDescriptorHandler } from '#domain/providers/npm';
import type { KeyDictionary } from '#domain/utils';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import Fixtures from './npmJsonParser.fixtures';

const complexTypeHandlers: KeyDictionary<JsonPackageTypeHandler> = {
  "version": createVersionDescFromJsonNode,
  "path": createPathDescFromJsonNode,
  "repository": createRepoDescFromJsonNode
};

export const extractPackageDependenciesFromJsonTests = {

  [test.title]: parsePackagesJson.name,

  "case $i: matches json packageManager": [
    Fixtures.matchesPackageManagerExpressions,
    Fixtures.matchesPackageManagerShaExpressions,
    Fixtures.matchesPackageManagerShaPrereleaseExpressions,
    (testFixture: any) => {
      const includePropNames = ['packageManager'];

      const testOptions: JsonParserOptions = {
        includePropNames,
        complexTypeHandlers,
        customDescriptorHandler
      };

      const results = parsePackagesJson(
        JSON.stringify(testFixture.test),
        testOptions
      );

      assert.deepEqual(results, testFixture.expected);
    },
  ],
};