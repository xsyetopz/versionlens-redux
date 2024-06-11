import { UpdateableFactory, getPreReleaseSuggestions } from '#domain/packages';
import { test } from 'mocha-ui-esm';
import { deepEqual, ok } from 'node:assert';

export const getPreReleaseSuggestionsTests = {

  [test.title]: getPreReleaseSuggestions.name,

  beforeEach: function (this: any) {
    this.testPrereleases = [
      '1.0.0-alpha',
      '1.0.1-alpha',
      '1.2.0-alpha',
      '1.2.0-dev',
      '1.2.0-beta'
    ]
  },

  'case $i: returns an empty array when a version is later than any prerelease': [
    '2.0.0-alpha',
    '2.0.0-dev', ,
    '2.0.0-beta',
    function (this: any, testVersion: string) {
      const actual = getPreReleaseSuggestions(testVersion, this.testPrereleases)
      ok(actual.length === 0);
    }
  ],

  'returns latest suggestions ordered by latest version first': function (this: any) {
    const testVersion = '~1.0.0-alpha'
    const expected = [
      UpdateableFactory.createTaggedPreleaseUpdateable('beta', '1.2.0-beta'),
      UpdateableFactory.createTaggedPreleaseUpdateable('dev', '1.2.0-dev'),
      UpdateableFactory.createTaggedPreleaseUpdateable('alpha', '1.2.0-alpha')
    ]

    // test
    const actual = getPreReleaseSuggestions(testVersion, this.testPrereleases)

    // assert
    deepEqual(actual, expected);
  }

}