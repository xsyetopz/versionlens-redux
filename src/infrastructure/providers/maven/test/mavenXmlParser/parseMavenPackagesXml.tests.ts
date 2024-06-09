import {
  extractReposUrlsFromXml,
  getVersionsFromPackageXml,
  parseMavenPackagesXml
} from 'infrastructure/providers/maven';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import Fixtures from './parseMavenPackagesXml.fixtures';

type TestContext = {
  test: {
    title: string
  }
}

export const parseMavenPackagesXmlTests = {

  [test.title]: parseMavenPackagesXml.name,

  'returns empty when no matches found with "$1"': [
    [[], ''],
    [["non-dependencies"], Fixtures.parseMavenPackagesXml.test],
    function (this: TestContext, testIncludeNames: Array<string>, expected: string) {
      // test
      const actual = parseMavenPackagesXml(expected, testIncludeNames);

      // assert
      assert.equal(actual.length, 0);
    }
  ],

  "extracts packages from maven xml": () => {
    // setup
    const includeNames = [
      "project.version",
      "project.parent",
      "project.dependencies.dependency",
    ];

    // test
    const actual = parseMavenPackagesXml(
      Fixtures.parseMavenPackagesXml.test,
      includeNames
    );

    // assert
    assert.deepEqual(actual, Fixtures.parseMavenPackagesXml.expected);
  },

  "extracts repositories from mvn cli xml": () => {
    // test
    const actual = extractReposUrlsFromXml(Fixtures.extractReposUrlsFromXml.test);

    // assert
    assert.deepEqual(actual, Fixtures.extractReposUrlsFromXml.expected);
  },

  "extracts versions from maven client xml": () => {
    // test
    const actual = getVersionsFromPackageXml(Fixtures.getVersionsFromPackageXml.test);

    // assert
    assert.deepEqual(actual, Fixtures.getVersionsFromPackageXml.expected);
  }

}