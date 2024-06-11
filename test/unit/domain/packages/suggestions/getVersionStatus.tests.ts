import { PackageStatusFactory, getVersionStatus, parseVersion } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import { deepEqual } from 'node:assert';

export const getVersionStatusTests = {

  [test.title]: getVersionStatus.name,

  beforeEach: function (this: any) {
    this.testReleases = ['5.1.1', '5.3.3', '5.4.5'];
    this.testPreReleases = ['5.3.3-dev.123', '5.5.0-dev.123'];
  },

  "returns InvalidRangeStatus when the range is invalid": () => {
    const testParsed = parseVersion('>1 <1', [], []);
    const actual = getVersionStatus(testParsed);
    deepEqual(actual, PackageStatusFactory.createInvalidRangeStatus());
  },

  "case $i: returns NoMatchStatus when no match found in releases or prereleases": [
    '1.1.*',
    '5.0.0',
    '6',
    '5.0.0-beta.341',
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, this.testPreReleases);
      const actual = getVersionStatus(testParsed);
      deepEqual(actual, PackageStatusFactory.createNoMatchStatus());
    }
  ],

  "case $i: returns SatisifiesLatestStatus when satisfies latest release": [
    '5.4.*',
    '5',
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, this.testPreReleases);
      const actual = getVersionStatus(testParsed);
      deepEqual(
        actual,
        PackageStatusFactory.createSatisifiesLatestStatus(
          testParsed.satisfiesVersion
        )
      );
    }
  ],

  "case $i: returns MatchesLatestStatus when is latest release": [
    '5.4.5',
    '~5.4.5',
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, this.testPreReleases);
      const actual = getVersionStatus(testParsed);
      deepEqual(
        actual,
        PackageStatusFactory.createMatchesLatestStatus(
          testParsed.satisfiesVersion
        )
      );
    }
  ],

  "case $i: returns MatchesLatestStatus when is latest prerelease": [
    '5.5.0-dev.123',
    '^5.5.0-dev.123',
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, this.testPreReleases);
      const actual = getVersionStatus(testParsed);
      deepEqual(
        actual,
        PackageStatusFactory.createMatchesLatestStatus(
          testParsed.satisfiesVersion
        )
      );
    }
  ],

  "case $i: returns FixedStatus when matches a release or prerelease": [
    '5.1.1',
    '5.3.3',
    '5.3.3-dev.123',
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, this.testPreReleases);
      const actual = getVersionStatus(testParsed);
      deepEqual(
        actual,
        PackageStatusFactory.createFixedStatus(
          testParsed.satisfiesVersion
        )
      );
    }
  ],

  "returns SatisifiesStatus when matches a release or prerelease": [
    '5.3.*',
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, this.testPreReleases);
      const actual = getVersionStatus(testParsed);
      deepEqual(
        actual,
        PackageStatusFactory.createSatisifiesStatus(
          testParsed.satisfiesVersion
        )
      );
    }
  ],

}