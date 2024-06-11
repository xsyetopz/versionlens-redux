import assert from 'node:assert';
import { VersionUtils } from '#domain/packages';

export const isFixedVersionTests = {

  title: VersionUtils.isFixedVersion.name,

  "returns false when not fixed": () => {
    const testVersions = [
      '~1.2.3',
      '^4.5.6-beta',
      '1.2.*',
      '>=1.2',
      '*',
    ]

    testVersions.forEach(testVersion => {
      const actual = VersionUtils.isFixedVersion(testVersion)
      assert.equal(actual, false);
    })
  },

  "returns true for fixed versions": () => {
    const testVersions = [
      '1.2.3',
      '4.5.6-beta',
    ]
    testVersions.forEach(testVersion => {
      const actual = VersionUtils.isFixedVersion(testVersion)
      assert.equal(actual, true);
    })
  },

}