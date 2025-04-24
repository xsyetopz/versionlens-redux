import { VersionUtils } from '#domain/packages';
import { equal } from 'assert';
import { test } from 'mocha-ui-esm';

const testReleases = [
  '0.9.0',
  '1.0.0',
  '2.0.0',
  '2.1.0',
  '2.1.1',
  '2.2.0',
  '3.3.1',
  '4.10.1',
  '4.100.0',
  '4.100.20',
];

export const findNextMajorTests = {

  [test.title]: VersionUtils.findNextMajor.name,

  "returns null when version array is empty": () => {
    const actual = VersionUtils.findNextMajor('2.2.0', []);
    equal(actual, null);
  },

  "case $i: returns null when no major exists": [
    '4.10.1',
    '4.100.0',
    '4.100.20',
    (testStartVersion: string) => {
      const actual = VersionUtils.findNextMajor(testStartVersion, testReleases);
      equal(actual, null);
    }
  ],

  "case $i: returns null when start version does not exist": [
    '1.1.0',
    '2.1.10',
    '3.0.1',
    '4.4.1',
    (testStartVersion: string) => {
      const actual = VersionUtils.findNextMajor(testStartVersion, testReleases);
      equal(actual, null);
    }
  ],

  "return null when invalid versions exist in the versions array": () => {
    const actual = VersionUtils.findNextMajor('2.0.0', ['2.0.0', 'ABC', '3.0.0']);
    equal(actual, null);
  },

  "case $i: handles loose versions": () => {
    const actual = VersionUtils.findNextMajor('2.0.0', ['2.0.0', '3.1.2ar']);
    equal(actual, '3');
  },

  "case $i: returns the next major from a versions array": [
    ['1.0.0', '2'],
    ['2.1.0', '3'],
    ['3.3.1', '4'],
    (testStartVersion: string, expected: string) => {
      const actual = VersionUtils.findNextMajor(testStartVersion, testReleases);
      equal(actual, expected);
    }
  ],

}