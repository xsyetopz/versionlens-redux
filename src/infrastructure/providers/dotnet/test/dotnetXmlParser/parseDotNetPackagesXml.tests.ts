import assert from 'assert';
import { parseDotNetPackagesXml } from 'infrastructure/providers/dotnet';
import { test } from 'mocha-ui-esm';
import Fixtures from './parseDotNetPackagesXml.fixtures';

type TestContext = {
  test: {
    title: string
  }
}

export const parsePackagesXmlTests = {

  [test.title]: parseDotNetPackagesXml.name,

  'returns empty when no matches found with "$1"': [
    [[], ''],
    [["non-dependencies"], Fixtures.parseDotNetPackagesXml.test],
    function (this: TestContext, testIncludeNames: Array<string>, expected: string) {
      // test
      const actual = parseDotNetPackagesXml(expected, testIncludeNames);

      // assert
      assert.equal(actual.length, 0);
    }
  ],

  "extracts packages from dotnet xml": () => {
    // setup
    const includeNames = [
      "Project.Sdk",
      "Project.ItemGroup.GlobalPackageReference",
      "Project.ItemGroup.PackageReference",
      "Project.ItemGroup.PackageVersion",
      "Project.ItemGroup.DotNetCliToolReference"
    ];

    // test
    const actual = parseDotNetPackagesXml(
      Fixtures.parseDotNetPackagesXml.test,
      includeNames
    );

    // assert
    assert.deepEqual(actual, Fixtures.parseDotNetPackagesXml.expected);
  }

}