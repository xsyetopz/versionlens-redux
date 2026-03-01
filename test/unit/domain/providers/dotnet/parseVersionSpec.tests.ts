import { parseVersionSpec } from '#domain/providers/dotnet';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';

type TestContext = {
  test: {
    title: string
  }
}

export const parseVersionSpecTests = {

  [test.title]: parseVersionSpec.name,

  'converts basic nuget range "$1" to semver range "$2"': [
    // basic
    ["1.0.0", "1.0.0"],
    ["(1.0.0,)", ">1.0.0"],
    ["[1.0.0]", "1.0.0"],
    ["(,1.0.0]", "<=1.0.0"],
    ["[1.0.0,2.0.0]", ">=1.0.0 <=2.0.0"],
    ["(1.0.0,2.0.0)", ">1.0.0 <2.0.0"],
    ["[1.0.0,2.0.0)", ">=1.0.0 <2.0.0"],
    function (this: TestContext, testDotNetSpec: string, expectedSemver: string) {
      this.test.title = this.test.title.replace("$1", testDotNetSpec);
      this.test.title = this.test.title.replace("$2", expectedSemver);
      const specTest = parseVersionSpec(testDotNetSpec);
      assert.equal(
        specTest.resolvedVersion,
        expectedSemver,
        `nuget range did not convert "${testDotNetSpec}" to "${expectedSemver}"`
      );
    }
  ],

  'converts partial nuget range "$1" to semver range "$2"': [
    ["1", "1.0.0"],
    ["1.0", "1.0.0"],
    ["[1,2]", ">=1.0.0 <=2.0.0"],
    ["(1,2)", ">1.0.0 <2.0.0"],
    function (this: TestContext, testDotNetSpec: string, expectedSemver: string) {
      this.test.title = this.test.title.replace("$1", testDotNetSpec);
      this.test.title = this.test.title.replace("$2", expectedSemver);
      const actual = parseVersionSpec(testDotNetSpec);
      assert.equal(
        actual.resolvedVersion,
        expectedSemver,
        `nuget range did not convert "${testDotNetSpec}" to "${expectedSemver}"`
      );
    }
  ],

  'returns null for invalid nuget range "$1"': [
    ["1."],
    ["1.0."],
    ["s.2.0"],
    ["beta"],
    function (this: TestContext, testDotNetSpec: string) {
      this.test.title = this.test.title.replace("$1", testDotNetSpec);
      const actual = parseVersionSpec(testDotNetSpec);
      assert.ok(
        !actual.spec,
        `Should not parse nuget range "${testDotNetSpec}"`
      );
    }
  ],

  'converts nuget floating range "$1" to semver range "$2"': [
    ["1.*", ">=1.0.0-0 <2.0.0-0"],
    ["1.0.*", ">=1.0.0-0 <1.1.0-0"],
    function (this: TestContext, testDotNetSpec: string, expectedSemver: string) {
      this.test.title = this.test.title.replace("$1", testDotNetSpec);
      this.test.title = this.test.title.replace("$2", expectedSemver);
      const actual = parseVersionSpec(testDotNetSpec);
      assert.equal(
        actual.resolvedVersion,
        expectedSemver,
        `nuget floating range did not convert "${testDotNetSpec}" to "${expectedSemver}"`
      );
    }
  ],

  'No nulls from "$1" valid notation': [
    // spec https://docs.microsoft.com/en-us/nuget/create-packages/dependency-versions#version-ranges
    ["1.0.0"],
    ["(1.0.0,)"],
    ["[1.0.0]"],
    ["(,1.0.0]"],
    ["(,1.0.0)"],
    ["[1.0.0,2.0.0]"],
    ["(1.0.0,2.0.0)"],
    ["[1.0.0,2.0.0)"],
    ["(1.0.0)"],
    function (this: TestContext, testDotNetSpec: string) {
      this.test.title = this.test.title.replace("$1", testDotNetSpec);
      const actual = parseVersionSpec(testDotNetSpec);
      assert.ok(
        !!actual,
        `Could not parse nuget range "${testDotNetSpec}"`
      )
    }
  ],

  'returns null using invalid notation "$1"': [
    ["1."],
    ["1.0."],
    ["s.2.0"],
    ["beta"],
    function (this: TestContext, testDotNetSpec: string) {
      this.test.title = this.test.title.replace("$1", testDotNetSpec);
      const actual = parseVersionSpec(testDotNetSpec);
      assert.ok(!actual.spec, `Range did not return null for "${testDotNetSpec}"`);
    }
  ],

  'returns true for four segment versions': () => {
    const actual = parseVersionSpec("1.0.0.1");
    assert.ok(actual.spec && actual.spec.hasFourSegments);
  }

}