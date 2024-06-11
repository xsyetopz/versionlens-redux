import assert from 'node:assert';
import { VersionUtils } from '#domain/packages';
import { test } from 'mocha-ui-esm';

const testPreleases = [
  '2.0.0-beta.1',
  '12.0.0-next.1',
  '12.0.0-dev',
];

export const filterPrereleaseTagsTests = {

  [test.title]: VersionUtils.filterPrereleaseTags.name,

  "returns all prereleases when tag filters are empty": () => {
    const testFilterTags: Array<string> = [];
    const actual = VersionUtils.filterPrereleaseTags(testPreleases, testFilterTags);
    assert.equal(actual.length, testPreleases.length);
    assert.deepEqual(actual, testPreleases);
  },

  "returns filtered prereleases based on tag filters": () => {
    const expectedPreReleases = ["2.0.0-beta.1"]
    const testFilterTags = ["beta"];

    const actual = VersionUtils.filterPrereleaseTags(testPreleases, testFilterTags);
    assert.equal(actual.length, 1);
    assert.deepEqual(actual, expectedPreReleases);
  },

  "returns empty prereleases when prereleases are empty": () => {
    const testFilterTags = ["beta"];
    const actual = VersionUtils.filterPrereleaseTags([], testFilterTags);
    assert.equal(actual.length, 0);
  },

}