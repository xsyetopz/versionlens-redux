import {
  createPathDescFromJsonNode,
  createRepoDescFromJsonNode,
  createVersionDescFromJsonNode,
  parsePackagesJson,
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler
} from '#domain/packages';
import { KeyDictionary } from 'domain/utils';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { customDescriptorHandler } from '../../src/parser/customDescriptorHandler';
import Fixtures from './npmJsonParser.fixtures';

const complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler> = {
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

      const testOptions: TJsonPackageParserOptions = {
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