import assert from 'node:assert';
import { PackageDependency, hasPackageDepsChanged } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import Fixtures from './hasPackageDepsChanged.fixtures';

export const hasPackageDepsChangedTests = {

  [test.title]: hasPackageDepsChanged.name,

  "returns false when original and changed are the same for $1": [
    ["no entries", []],
    ["single entries", Fixtures.single],
    ["multiple entries", Fixtures.multiple],
    function (caseTitle: string, testDeps: PackageDependency[]) {
      const actual = hasPackageDepsChanged(testDeps, testDeps);
      assert.equal(actual, false);
    }
  ],

  "returns false when only $1 name range has changed": [
    ["original single entry", Fixtures.singleWithDiffNameRange, Fixtures.single],
    ["changed single entry", Fixtures.single, Fixtures.singleWithDiffNameRange],
    ["original multiple entries", Fixtures.multipleWithDiffNameRange, Fixtures.multiple],
    ["changed multiple entries", Fixtures.multiple, Fixtures.multipleWithDiffNameRange],
    function (caseTitle: string, testOriginal: PackageDependency[], testChanged: PackageDependency[]) {
      const actual = hasPackageDepsChanged(testOriginal, testChanged);
      assert.equal(actual, false);
    }
  ],

  "returns false when only $1 version range has changed": [
    ["original single entry", Fixtures.singleWithDiffVersionRange, Fixtures.single],
    ["changed single entry", Fixtures.single, Fixtures.singleWithDiffVersionRange],
    ["original multiple entries", Fixtures.multipleWithDiffVersionRange, Fixtures.multiple],
    ["changed multiple entries", Fixtures.multiple, Fixtures.multipleWithDiffVersionRange],
    function (caseTitle: string, testOriginal: PackageDependency[], testChanged: PackageDependency[]) {
      const actual = hasPackageDepsChanged(testOriginal, testChanged);
      assert.equal(actual, false);
    }
  ],

  "returns true when $1 items then changed": [
    ["original has less", Fixtures.single, Fixtures.multiple],
    ["changed has less", Fixtures.multiple, Fixtures.single],
    function (caseTitle: string, testOriginal: PackageDependency[], testChanged: PackageDependency[]) {
      const actual = hasPackageDepsChanged(testOriginal, testChanged);
      assert.equal(actual, true);
    }
  ],

  "returns true when version has changed for $1": [
    ["original single entries", Fixtures.singleWithDiffVersion, Fixtures.single],
    ["changed single entries", Fixtures.single, Fixtures.singleWithDiffVersion],
    ["original multiple entries", Fixtures.multipleWithDiffVersion, Fixtures.multiple],
    ["changed multiple entries", Fixtures.multiple, Fixtures.multipleWithDiffVersion],
    function (caseTitle: string, testOriginal: PackageDependency[], multipleWithDiffVersionRange: PackageDependency[]) {
      const actual = hasPackageDepsChanged(testOriginal, multipleWithDiffVersionRange);
      assert.equal(actual, true);
    }
  ],

  "returns false when has ignoresChanges descriptors": function () {
    const actual = hasPackageDepsChanged(Fixtures.ignoresChanges, Fixtures.multiple);
    assert.equal(actual, false);
  }

};