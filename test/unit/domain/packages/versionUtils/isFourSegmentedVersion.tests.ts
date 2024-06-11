import assert from 'node:assert';
import { VersionUtils } from '#domain/packages';
import { test } from 'mocha-ui-esm';

export const isFourSegmentedVersionTests = {

  [test.title]: VersionUtils.isFourSegmentedVersion.name,

  "returns false for semver versions $1": [
    '~1.2.3',
    '^4.5.6-beta',
    '1.2.*',
    '>=1.2',
    '*',
    (testVersion: string) => {
      const actual1 = VersionUtils.isFourSegmentedVersion(testVersion)
      assert.equal(actual1, false);
    }
  ],

  "returns true for four segment versions $1": [
    '1.2.3.4',
    '1.0.1.1',
    '1.0.0.10',
    (testVersion: string) => {
      const actual1 = VersionUtils.isFourSegmentedVersion(testVersion)
      assert.ok(actual1);
    }
  ],

}