import { parseDotNetPackagesXml } from '#domain/providers/dotnet';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
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
    [["non-dependencies"], Fixtures.parsesItemGroupPackages.test],
    function (this: TestContext, testIncludeNames: Array<string>, expected: string) {
      // test
      const actual = parseDotNetPackagesXml(expected, testIncludeNames);

      // assert
      assert.equal(actual.length, 0);
    }
  ],

  "case $i: extracts packages from dotnet xml": [
    Fixtures.parsesItemGroupPackages,
    Fixtures.parsesPropertyGroupVersions,
    (fixutre: any) => {
      // setup
      const includeNames = [
        "Project.PropertyGroup.AssemblyVersion",
        "Project.PropertyGroup.Version",
        "Project.Sdk",
        "Project.ItemGroup.GlobalPackageReference",
        "Project.ItemGroup.PackageReference",
        "Project.ItemGroup.PackageVersion",
        "Project.ItemGroup.DotNetCliToolReference"
      ];

      // test
      const actual = parseDotNetPackagesXml(
        fixutre.test,
        includeNames
      );

      // assert
      assert.deepEqual(actual, fixutre.expected);
    }
  ]

}