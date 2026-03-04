import { parsePackagesGoMod } from '../../../../../src/domain/providers/golang/goModParser';
import { test } from 'mocha-ui-esm';
import { deepEqual, equal } from 'node:assert';
import Fixtures from './goModParser.fixtures';

export const goModParserTests = {

  [test.title]: 'parsePackagesGoMod',

  "returns empty when file is empty": () => {
    const actual = parsePackagesGoMod('');
    equal(actual.length, 0);
  },

  "ignores pseudo versions": () => {
    const testText = `
      require k8s.io/utils v0.0.0-20230726121419-3b25d923346b
    `;
    const actual = parsePackagesGoMod(testText);
    equal(actual.length, 0);
  },

  "case $i: $1": [
    ["parsesGoMod", Fixtures.parsesGoMod],
    ["smoke", Fixtures.smoke],
    (name: string, fixture: any) => {
      const actual = parsePackagesGoMod(fixture.test);
      deepEqual(actual, fixture.expected);
    }
  ]

};
