import assert from 'node:assert';
import { VersionUtils } from '#domain/packages';

const testVersions = [
  '1.0.0',
  '2.0.0',
  '2.0.0-beta.1',
  '2.0.0.1',
  '9.5.12',
  '11.1.9',
  '11.1.9.1',
  '12.0.0-next.1',
]

export const removeFourSegmentVersionsFromArrayTests = {

  title: VersionUtils.removeFourSegmentVersionsFromArray.name,

  "returns versions when no matches found": () => {
    const expected = [
      '1.0.0',
      '2.0.0',
    ]
    const results = VersionUtils.removeFourSegmentVersionsFromArray(expected);
    assert.equal(results.length, expected.length);
    expected.forEach((expectedVersion, index) => {
      assert.equal(results[index], expectedVersion);
    })
  },

  "returns versions without four segment versions": () => {
    const expected = [
      '1.0.0',
      '2.0.0',
      '2.0.0-beta.1',
      '9.5.12',
      '11.1.9',
      '12.0.0-next.1',
    ]
    const results = VersionUtils.removeFourSegmentVersionsFromArray(testVersions);
    assert.equal(results.length, expected.length);
    expected.forEach((expectedVersion, index) => {
      assert.equal(results[index], expectedVersion);
    })
  }

}