import { parseVersion } from 'domain/packages';
import { test } from 'mocha-ui-esm';
import { equal, notEqual, ok } from 'node:assert';

export const parseVersionTests = {

  [test.title]: parseVersion.name,

  beforeEach: function (this: any) {
    this.testReleases = ['5.3.3', '5.4.3', '5.4.4', '5.4.5'];
    this.testPrereleases = ['5.3.3-dev.1234', '5.4.3-beta.4567', '5.4.4-rc.6789', '5.4.5-next.1452'];
  },

  isPreRelease: {
    'case $i: is true for prerelease versions': [
      '1.2.3-pre',
      '1.2.*-pre',
      '1.2.*-pre.123',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        ok(actual.isPreRelease);
      }
    ],
    'case $i: is false for release versions': [
      '1.0.0',
      '1.0.*',
      '*',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        ok(actual.isPreRelease === false, testVersion);
      }
    ],
  },

  isFixedVersion: {
    'case $i: is true for fixed versions': [
      '1.2.3',
      '1.2.3-pre',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        ok(actual.isFixedVersion);
      }
    ],
    'case $i: is false for ranged versions': [
      '1.0.*',
      '1.*',
      '*',
      '1.0.*-pre',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        ok(actual.isFixedVersion === false, testVersion);
      }
    ],
  },

  isRangeVersion: {
    'case $i: is true for ranged versions': [
      '1.0.*',
      '1.*',
      '*',
      '1.0.*-pre',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        ok(actual.isRangeVersion);
      }
    ],
    'case $i: is false for fixed versions': [
      '1.2.3',
      '1.2.3-pre',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        ok(actual.isRangeVersion === false, testVersion);
      }
    ],
  },

  minVersion: {
    'case $i: returns minVersion for ranged versions': [
      ['1.0.*', '1.0.0'],
      ['1.*', '1.0.0'],
      ['*', '0.0.0'],
      ['> 2.0.0', '2.0.1'],
      ['> 2.0', '2.1.0'],
      ['> 2', '3.0.0'],
      ['1.0.*-pre', '1.0.0'],
      (testVersion: string, expected: string) => {
        const actual = parseVersion(testVersion, [], []);
        equal(actual.minVersion, expected);
      }
    ],
    'case $i: not set for fixed versions': [
      '1.0.0',
      '1.0.0-pre',
      (testVersion: string) => {
        const actual = parseVersion(testVersion, [], []);
        equal(actual.minVersion, undefined);
      }
    ],
  },

  satisfiesVersion: {
    'case $i: returns satisifedVersion for matching fixed versions': [
      '5.3.3',
      '5.4.3',
      '5.4.4',
      '5.4.5',
      '5.3.3-dev.1234',
      '5.4.3-beta.4567',
      '5.4.4-rc.6789',
      '5.4.5-next.1452',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
        equal(actual.satisfiesVersion, testVersion);
      }
    ],
    'case $i: returns satisifedVersion for matching ranged versions': [
      ['5.3.*', '5.3.3'],
      ['5.*', '5.4.5'],
      ['*', '5.4.5'],
      ['5.3.*-pre', '5.3.3'],
      ['5.*.*-pre', '5.4.5'],
      function (this: any, testVersion: string, expected: string) {
        const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
        equal(actual.satisfiesVersion, expected);
      }
    ],
    'case $i: not set for unsatisfied fixed versions': [
      '5.3.1',
      '5.4.0',
      '5.4.0-pre',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, []);
        equal(actual.satisfiesVersion, undefined);
        notEqual(actual.satisfiesVersion, testVersion);
      }
    ],
    'case $i: not set for unsatisfied ranged versions': [
      '5.6.*',
      '6.*',
      '6',
      '5.6.*-pre',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, []);
        equal(actual.satisfiesVersion, undefined);
        notEqual(actual.satisfiesVersion, testVersion);
      }
    ],
  },

  hasRangeUpdate: {
    'case $i: is true when has satisfiesVersion and satisfiesVersion not equal to minVersion': [
      '5.4.*',
      '5.*',
      '5',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, []);
        equal(actual.satisfiesVersion, '5.4.5');
        notEqual(actual.satisfiesVersion, actual.minVersion);
        ok(actual.hasRangeUpdate);
      }
    ],
    'case $i: is false when has satisfiesVersion and satisfiesVersion equal to minVersion': [
      ['~5.3.3', '5.3.3'],
      ['~5.4.5', '5.4.5'],
      function (this: any, testVersion: string, expectedSatisfiesVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, []);
        equal(actual.satisfiesVersion, expectedSatisfiesVersion);
        equal(actual.minVersion, actual.satisfiesVersion);
        ok(actual.hasRangeUpdate === false);
      }
    ],
  },

  isLatest: {
    'case $i: is true when equals latest version': [
      '5.4.5',
      '5.4.*',
      '5.4.*-pre',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
        ok(actual.isLatest);
        equal(actual.latestRelease, this.testReleases[this.testReleases.length - 1]);
      }
    ],
    'case $i: is false for non matching latest versions': [
      '1.2.3',
      '5.4.4',
      '5.4.5-pre',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
        ok(actual.isLatest === false);
        equal(actual.latestRelease, this.testReleases[this.testReleases.length - 1]);
      }
    ],
  },

  isLatestPrerelease: {
    'is true when equals latest prerelease version': function (this: any) {
      const testVersion = '5.4.5-next.1452';
      const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
      ok(actual.isPreRelease);
      equal(actual.latestPreRelease, this.testPrereleases[this.testReleases.length - 1]);
    },
    'case $i: is false for non matching latest prerelease versions': [
      '5.4.5-pre',
      '5.6.0-pre',
      '5.4.*-pre',
      function (this: any, testVersion: string) {
        const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
        ok(actual.isLatestPreRelease === false);
        equal(actual.latestPreRelease, undefined);
      }
    ],
  },

  hasInvalidRange: {
    'is true when isRangeVersion and minVersion is null': function (this: any) {
      const testVersion = '>1 <1';
      const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
      ok(actual.isRangeVersion);
      equal(actual.minVersion, undefined);
      ok(actual.hasInvalidRange);
    },
    'is false when isRangeVersion and minVersion is set': function (this: any) {
      const testVersion = '>1 <3';
      const actual = parseVersion(testVersion, this.testReleases, this.testPrereleases);
      ok(actual.isRangeVersion);
      equal(actual.minVersion, '2.0.0');
      ok(actual.hasInvalidRange === false);
    },
  }

}