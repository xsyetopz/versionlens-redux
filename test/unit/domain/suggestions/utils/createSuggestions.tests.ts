import assert from 'node:assert';
import {
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  TPackageSuggestion,
  createSuggestions
} from 'domain/packages';
import { test } from 'mocha-ui-esm';
import Fixtures from './createSuggestions.fixtures';

export const CreateSuggestionsTests = {

  [test.title]: createSuggestions.name,

  "returns nomatch": {

    "when releases and prereleases are empty": () => {
      const expected = [
        <TPackageSuggestion>{
          name: SuggestionStatusText.NoMatch,
          version: '',
          type: SuggestionTypes.status
        }
      ]

      const testRange = '*'
      const testReleases: Array<string> = []
      const testPrereleases: Array<string> = []
      const results = createSuggestions(
        testRange,
        testReleases,
        testPrereleases
      );
      assert.equal(results.length, expected.length);
      assert.equal(results[0].name, expected[0].name);
      assert.equal(results[0].version, expected[0].version);
      assert.equal(results[0].type, expected[0].type);
    },

    "when releases or prereleases do not contain a matching version": () => {

      const expected = [
        <TPackageSuggestion>{
          name: SuggestionStatusText.NoMatch,
          category: SuggestionCategory.NoMatch,
          version: '',
          type: SuggestionTypes.status
        },
        <TPackageSuggestion>{
          name: SuggestionStatusText.UpdateLatest,
          category: SuggestionCategory.Updateable,
          version: '1.0.0',
          type: SuggestionTypes.release
        }
      ]

      const testRange = '2.0.0'
      const testReleases = ['1.0.0']
      const testPrereleases = ['1.1.0-alpha.1']
      const results = createSuggestions(
        testRange,
        testReleases,
        testPrereleases
      );
      assert.deepEqual(results, expected);
    },
  },

  "when has dist tag suggestion": {
    "and version has no match": {
      "returns 'no match' with latest dist tag suggestion": () => {
        // setup
        const testDistTagLatest = '4.0.0-next';

        const expected = [
          <TPackageSuggestion>{
            name: SuggestionStatusText.NoMatch,
            category: SuggestionCategory.NoMatch,
            version: '',
            type: SuggestionTypes.status
          },
          <TPackageSuggestion>{
            name: SuggestionStatusText.UpdateLatestPrerelease,
            category: SuggestionCategory.Updateable,
            version: '4.0.0-next',
            type: SuggestionTypes.prerelease
          }
        ]

        const testRange = '4.0.0'
        const testReleases = ['0.0.5', '0.0.6']
        const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']

        // test
        const results = createSuggestions(
          testRange,
          testReleases,
          testPrereleases,
          testDistTagLatest
        );

        // assert
        assert.deepEqual(results, expected);
      },
    },
    "version matches dist tag": {
      "returns 'latest'": () => {
        // setup
        const testDistTagVersion = '5.0.0';

        const expected = [
          <TPackageSuggestion>{
            name: SuggestionStatusText.Latest,
            category: SuggestionCategory.Latest,
            version: '5.0.0',
            type: SuggestionTypes.status
          }
        ]

        const testReleases = ['0.0.5', '2.0.0', '5.0.0']
        const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']
        const testRange = testDistTagVersion

        // test
        const results = createSuggestions(
          testRange,
          testReleases,
          testPrereleases,
          testDistTagVersion
        );

        // assert
        assert.deepEqual(results, expected);
      }
    }
  },

  "when version is fixed": {
    "has no match": {
      "returns 'no match' with latest suggestions": () => {
        // setup
        const testRange = '0.5.0'
        const testReleases = ['1.0.0']
        const testPrereleases = ['1.1.0-alpha.1']

        // test
        const results = createSuggestions(
          testRange,
          testReleases,
          testPrereleases
        );

        // assert
        assert.deepEqual(results, Fixtures.fixedNoMatchWithLatestSuggestions);
      }
    },
    "is the latest release": {
      "returns 'latest' with latest prerelease suggestions": () => {
        // setup
        const testVersion = '3.0.0';
        const testReleases = ['1.0.0', '2.0.0', '2.1.0', testVersion]
        const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']

        // test
        const results = createSuggestions(
          testVersion,
          testReleases,
          testPrereleases
        );

        // assert
        assert.deepEqual(results, Fixtures.fixedIsLatestWithPrereleaseSuggestions);
        assert.equal(results[0].version, testVersion);
      },
      "returns 'latest' with no suggestions": () => {
        // setup
        const testVersion = '3.0.0';
        const testReleases = ['1.0.0', '2.0.0', '2.1.0', testVersion]
        const testPrereleases = ['1.1.0-alpha.1', '3.0.0-next']

        // test
        const results = createSuggestions(
          testVersion,
          testReleases,
          testPrereleases
        );

        // assert
        assert.deepEqual(results, Fixtures.fixedIsLatestNoSuggestions);
        assert.equal(results[0].version, testVersion);
      }
    }
  },

  'fixed with latest, minor and patch': {
    "$i: returns 'fixed' with latest, minor and patch suggestions": [
      ['1.1.1'],
      (testRange: string) => {
        // setup
        const fixedVersion = '1.1.1';
        const testReleases = [
          '1.1.0',
          '1.1.1',
          '1.1.2',
          '1.2.0',
          '1.2.2',
          '2.0.0',
          '2.2.2',
        ];
        const testPrereleases = [];

        // test
        const results = createSuggestions(
          testRange,
          testReleases,
          testPrereleases
        );

        // assert
        assert.deepEqual(results, Fixtures.fixedWithSuggestions);
        assert.equal(results[0].version, fixedVersion);
      },
    ],
  },

  "when version is a range": {
    "has no match": {
      "returns 'no match' with latest suggestion": () => {
        // setup
        const testRange = '>2.0.0 <3.0.0'
        const testReleases = ['1.0.0', '2.0.0']
        const testPrereleases = ['1.1.0-alpha.1']

        // test
        const results = createSuggestions(
          testRange,
          testReleases,
          testPrereleases
        );

        // assert
        assert.deepEqual(results, Fixtures.rangeNoMatchWithLatestSuggestions);
      }
    },

    "matches the latest release": {
      "$i: returns 'latest' with latest prerelease suggestions": [
        ['>=3'],
        ['^3'],
        ['3.*'],
        ['^3.0.0'],
        ['>=3.0.* < 4'],
        (testRange: string) => {
          // setup
          const latestVersion = '3.0.0';
          const testReleases = ['1.0.0', '2.0.0', '2.1.0', latestVersion]
          const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']

          // test
          const results = createSuggestions(
            testRange,
            testReleases,
            testPrereleases
          );

          // assert
          assert.deepEqual(results, Fixtures.rangeSatisfiesLatest);
          assert.equal(results[0].version, latestVersion);
        }
      ],
    },
    "satisifes the latest release": {
      "$i: returns 'satisifes latest' with latest prerelease suggestions": [
        ['>=2'],
        ['>=2 <=5'],
        (testRange: string) => {
          // setup
          const latestVersion = '3.0.0';
          const testReleases = ['1.0.0', '2.0.0', '2.1.0', latestVersion]
          const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']

          // test
          const results = createSuggestions(
            testRange,
            testReleases,
            testPrereleases
          );

          // assert
          assert.deepEqual(results, Fixtures.latestWithinRange);
          assert.equal(results[0].version, latestVersion);
        }
      ],
    },

    "satisfies an update within the range": {
      "$i: returns 'satisfies' with update suggestion": [
        ['>=2 <3'],
        ['>=1.2 <2.2.*'],
        (testRange: string) => {
          // setup
          const satisfiesVersion = '2.1.0';
          const testReleases = ['1.0.0', '2.0.0', '2.1.0', '3.0.0']
          const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']

          // test
          const results = createSuggestions(
            testRange,
            testReleases,
            testPrereleases
          );

          // assert
          assert.deepEqual(results, Fixtures.rangeSatisfiesUpdateAndSuggestsLatest);
          assert.equal(results[0].version, satisfiesVersion);
        }
      ],
    },

    'satisfies ~ range with update suggestions': {
      "$i: returns 'satisfies' with update, latest, minor suggestions": [
        ['~1.1'],
        ['~1.1.1'],
        (testRange: string) => {
          // setup
          const satisfiesVersion = '1.1.2';
          const testReleases = [
            '1.1.0',
            '1.1.1',
            '1.1.2',
            '1.2.0',
            '1.2.2',
            '2.0.0',
            '2.2.2',
          ];
          const testPrereleases = [];

          // test
          const results = createSuggestions(
            testRange,
            testReleases,
            testPrereleases
          );

          // assert
          assert.deepEqual(
            results,
            Fixtures.rangeSatisfiesTildeRangeWithUpdateSuggestions
          );
          assert.equal(results[0].version, satisfiesVersion);
        },
      ],
    },

    'satisfies ^ range with update suggestions': {
      "$i: returns 'satisfies' with update, latest suggestions": [
        ['^1.1'],
        ['^1.1.1'],
        (testRange: string) => {
          // setup
          const satisfiesVersion = '1.2.2';
          const testReleases = [
            '1.1.0',
            '1.1.1',
            '1.1.2',
            '1.2.0',
            '1.2.2',
            '2.0.0',
            '2.2.2',
          ];
          const testPrereleases = [];

          // test
          const results = createSuggestions(
            testRange,
            testReleases,
            testPrereleases
          );

          // assert
          assert.deepEqual(
            results,
            Fixtures.rangeSatisfiesCaretRangeWithUpdateSuggestions
          );
          assert.equal(results[0].version, satisfiesVersion);
        },
      ],
    },

    "satisfies maximum range": {
      "$i: returns 'satisfies' with latest suggestion": [
        ['^2.1.0'],
        (testRange: string) => {
          // setup
          const satisfiesVersion = '2.1.0';
          const testReleases = ['1.0.0', '2.0.0', '2.1.0', '3.0.0']
          const testPrereleases = ['1.1.0-alpha.1', '4.0.0-next']

          // test
          const results = createSuggestions(
            testRange,
            testReleases,
            testPrereleases
          );

          // assert
          assert.deepEqual(results, Fixtures.rangeSatisfiesMaxAndSuggestsLatest);
          assert.equal(results[0].version, satisfiesVersion);
        }
      ],
    }
  }
}