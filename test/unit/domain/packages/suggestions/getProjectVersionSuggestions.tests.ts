import { SuggestionIncrements, UpdateableFactory, getProjectVersionSuggestions } from 'domain/packages';
import { test } from 'mocha-ui-esm';
import { deepEqual, ok } from 'node:assert';

export const getProjectVersionSuggestionsTests = {

  [test.title]: getProjectVersionSuggestions.name,

  'case $i: handles empty or invalid version strings': [
    '',
    'invalid',
    (testVersion: string) => {
      const expected = [
        UpdateableFactory.createNextMaxUpdateable(
          '1.0.0',
          SuggestionIncrements.major
        ),
        UpdateableFactory.createNextMaxUpdateable(
          '0.1.0',
          SuggestionIncrements.minor
        ),
        UpdateableFactory.createNextMaxUpdateable(
          '0.0.1',
          SuggestionIncrements.patch
        )
      ];

      // test
      const actual = getProjectVersionSuggestions(testVersion);

      // assert
      ok(actual.length === expected.length);
      deepEqual(actual, expected);
    }
  ],

  'returns release increment suggestions': () => {
    const testVersion = '1.0.0';
    const expected = [
      UpdateableFactory.createNextMaxUpdateable(
        '2.0.0',
        SuggestionIncrements.major
      ),
      UpdateableFactory.createNextMaxUpdateable(
        '1.1.0',
        SuggestionIncrements.minor
      ),
      UpdateableFactory.createNextMaxUpdateable(
        '1.0.1',
        SuggestionIncrements.patch
      )
    ];

    // test
    const actual = getProjectVersionSuggestions(testVersion);

    // assert
    ok(actual.length === expected.length);
    deepEqual(actual, expected);
  },

  'returns prerelease increment suggestions': () => {
    const testVersion = '1.0.0-pre';
    const expected = [
      UpdateableFactory.createNextMaxUpdateable(
        '1.0.0',
        SuggestionIncrements.release
      ),
      UpdateableFactory.createNextMaxUpdateable(
        '1.0.0-pre.0',
        SuggestionIncrements.prerelease
      )
    ];

    // test
    const actual = getProjectVersionSuggestions(testVersion);

    // assert
    ok(actual.length === expected.length);
    deepEqual(actual, expected);
  },

}