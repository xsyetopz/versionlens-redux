import assert from 'node:assert';
import { VersionUtils } from '#domain/packages';

const testVersions = [
  '1.0.0.5',
  '2.0.0.33333',
  '2.0.0-beta.1',
  '9.5.12',
  '11.1.9',
  '12.0.0-next.1',
  '~master',
]

export const filterSemverVersionsTests = {

  title: VersionUtils.filterSemverVersions.name,

  "returns empty when versions is empty": () => {
    const results = VersionUtils.filterSemverVersions([]);
    assert.equal(results.length, 0);
  },

  "returns empty when no matches found": () => {
    const results = VersionUtils.filterSemverVersions(['1.2.3.4', '5.6.7.8']);
    assert.equal(results.length, 0);
  },

  "returns semver versions only": () => {
    const expected = [
      '2.0.0-beta.1',
      '9.5.12',
      '11.1.9',
      '12.0.0-next.1',
    ]
    const results = VersionUtils.filterSemverVersions(testVersions);
    assert.equal(results.length, expected.length);
    expected.forEach((expectedValue, index) => {
      assert.equal(results[index], expectedValue);
    })
  },

}