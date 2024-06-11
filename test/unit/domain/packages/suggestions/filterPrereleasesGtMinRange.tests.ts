import { filterPrereleasesGtMinRange } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';

export const filterPrereleasesGtMinRangeTests = {

  [test.title]: filterPrereleasesGtMinRange.name,

  beforeEach: function (this: any) {
    this.testPrereleases = [
      '2.0.0-preview1.12141.1',
      '2.0.0-preview2.45112.2',
      '2.0.0-preview3.13311.9',
      '2.0.0-preview4.17421.6',
      '2.1.0-preview1-final',
      '2.1.0-dev.1',
      '2.1.0-dev.2',
      '2.1.0-dev.3',
      '2.1.0-beta1',
      '2.1.0-beta2',
      '2.1.0-beta3',
      '2.1.0-rc.1',
      '2.1.0-rc.2',
      '2.1.0-rc.3',
      '2.5.0-dev.1',
    ];
  },

  "returns empty when no matches found": () => {
    const actual = filterPrereleasesGtMinRange('*', []);
    assert.equal(actual.length, 0);
  },

  "case $i: handles non semver versions without error": [
    '2.1.0.RELEASE',
    '2.1.0.3',
    function (this: any, testVersion: string) {
      const actual = filterPrereleasesGtMinRange(testVersion, this.testPrereleases);
      assert.deepEqual(actual.length, 0, testVersion);
    }
  ],

  "groups prereleases by name": function (this: any) {
    const expected = [
      '2.1.0-preview1-final',
      '2.1.0-beta3',
      '2.1.0-rc.3',
      '2.5.0-dev.1',
    ]
    // test
    const actual = filterPrereleasesGtMinRange('2.*', this.testPrereleases);

    // assert
    assert.equal(actual.length, expected.length);
    expected.forEach((expectedValue, index) => {
      assert.equal(actual[index], expectedValue);
    })
  },

  "case $i: returns empty when prereleases are <= specified versions": [
    // greater
    '3.*',
    '~2.6.1',
    '2.5.9',
    // equals
    '2.5.0-dev.1',
    function (this: any, testVersion: string) {
      const actual = filterPrereleasesGtMinRange(testVersion, this.testPrereleases);
      assert.deepEqual(actual.length, 0, testVersion);
    }
  ],

  "case $i: returns prereleases > specified versions": [
    '2.*',
    '~2.0.1',
    '2.0.9',
    function (this: any, testVersion: string) {
      const expected = [
        '2.1.0-preview1-final',
        '2.1.0-beta3',
        '2.1.0-rc.3',
        '2.5.0-dev.1',
      ]

      // test
      const actual = filterPrereleasesGtMinRange(testVersion, this.testPrereleases);

      // assert
      assert.deepEqual(actual, expected);
    }
  ],
}