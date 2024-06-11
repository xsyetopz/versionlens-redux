import assert from 'node:assert';
import { VersionUtils } from '#domain/packages';

const testVersions = [
  '1.0.0',
  '2.0.0',
  '2.0.0-beta.1',
  '9.5.12',
  '11.1.9',
  '12.0.0-next.1',
]

const expectedReleases = [
  '1.0.0',
  '2.0.0',
  '9.5.12',
  '11.1.9',
]

const expectedPrereleases = [
  '2.0.0-beta.1',
  '12.0.0-next.1',
]

export const splitReleasesFromArrayTests = {

  title: VersionUtils.splitReleasesFromArray.name,

  "returns empty when no matches found": () => {
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray([], []);
    assert.equal(releases.length, 0);
    assert.equal(prereleases.length, 0);
  },

  "returns mapped PackageNameVersion array": () => {
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      testVersions,
      []
    );

    assert.equal(releases.length, expectedReleases.length);
    assert.equal(prereleases.length, expectedPrereleases.length);

    expectedReleases.forEach((expectedVersion, index) => {
      assert.equal(releases[index], expectedVersion);
    })

    expectedPrereleases.forEach((expectedVersion, index) => {
      assert.equal(prereleases[index], expectedVersion);
    })
  },

  "filters prereleases": () => {

    const expectedPrereleases = [
      '2.0.0-beta.1'
    ]

    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      testVersions,
      ["beta"]
    );

    assert.equal(releases.length, expectedReleases.length);
    assert.equal(prereleases.length, expectedPrereleases.length);

    expectedReleases.forEach((expectedVersion, index) => {
      assert.equal(releases[index], expectedVersion);
    });

    expectedPrereleases.forEach((expectedVersion, index) => {
      assert.equal(prereleases[index], expectedVersion);
    });
  },

}