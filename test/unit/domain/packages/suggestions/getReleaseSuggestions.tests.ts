import { UpdateableFactory, getReleaseSuggestions, parseVersion } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import { deepEqual, ok } from 'node:assert';

export const getReleaseSuggestionsTests = {

  [test.title]: getReleaseSuggestions.name,

  beforeEach: function (this: any) {
    this.testReleases = [
      '3.0.0',
      '3.1.0',
      '4.0.0',
      '4.0.1',
      '4.1.10',
      '5.1.1',
      '5.2.0',
      '5.3.3',
      '5.4.5'
    ]
  },

  "case $i: suggests latest": [
    '5.4.0',
    '~5.4.4',
    '6', // no match
    function (this: any, testVersion: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, [])

      // test
      const actual = getReleaseSuggestions(testVersion, testParsed, this.testReleases)

      // assert
      ok(actual.length > 0)

      deepEqual(
        actual[actual.length - 1],
        UpdateableFactory.createLatestUpdateable(
          this.testReleases[this.testReleases.length - 1]
        )
      )
    }
  ],

  "case $i: suggests minor": [
    ['3.0.0', '3.1.0'],
    ['4.0.1', '4.1.10'],
    function (this: any, testVersion: string, expected: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, [])

      // test
      const actual = getReleaseSuggestions(testVersion, testParsed, this.testReleases)

      // assert
      ok(actual.length > 1)

      deepEqual(
        actual[actual.length - 1],
        UpdateableFactory.createNextMaxUpdateable(
          expected,
          'minor'
        )
      )
    }
  ],

  "case $i: suggests patch": [
    ['4.1.0', '4.1.10'],
    ['5.3.0', '5.3.3'],
    function (this: any, testVersion: string, expected: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, [])

      // test
      const actual = getReleaseSuggestions(testVersion, testParsed, this.testReleases)

      // assert
      ok(actual.length > 1)

      deepEqual(
        actual[actual.length - 1],
        UpdateableFactory.createNextMaxUpdateable(
          expected,
          'patch'
        )
      )
    }
  ],

  "case $i: suggests bump": [
    ['^4.1.0', '4.1.10'],
    ['~4.0.0', '4.0.1'],
    function (this: any, testVersion: string, expected: string) {
      const testParsed = parseVersion(testVersion, this.testReleases, [])

      // test
      const actual = getReleaseSuggestions(testVersion, testParsed, this.testReleases)

      // assert
      ok(actual.length > 1)

      deepEqual(
        actual[actual.length - 1],
        UpdateableFactory.createNextMaxUpdateable(
          expected,
          'bump'
        )
      )
    }
  ]

}