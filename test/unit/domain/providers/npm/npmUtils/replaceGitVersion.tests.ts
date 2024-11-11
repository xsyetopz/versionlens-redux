import {
  mapToSuggestionUpdate,
  PackageDependency,
  PackageResponse,
  PackageSourceType,
  SuggestionCategory,
  SuggestionTypes
} from '#domain/packages';
import { createDependencyRange, PackageDescriptor } from '#domain/parsers';
import { npmReplaceVersion } from '#domain/providers/npm';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';

export const npmReplaceVersionTests = {

  [test.title]: npmReplaceVersion.name,

  "handles #tag|commit|semver:": () => {
    const packageInfo: PackageResponse = {
      order: 0,
      providerName: 'testreplace',
      packageSource: PackageSourceType.Github,
      parsedDependency: <PackageDependency>{
        nameRange: createDependencyRange(0, 0),
        versionRange: createDependencyRange(1, 1),
        package: {
          path: 'packagepath',
          name: 'packagename',
          version: 'github:someRepo/someProject#semver:^2',
        },
        descriptors: new PackageDescriptor([]),
      },
      fetchedPackage: {
        name: 'packagename',
        version: '^2'
      },
      suggestion: {
        name: 'packagename',
        category: SuggestionCategory.Updateable,
        type: SuggestionTypes.release,
        version: '4.2.1'
      }
    }

    const expected = 'github:someRepo/someProject#semver:4.2.1'

    // NpmVersionUtils
    const actual = npmReplaceVersion(mapToSuggestionUpdate(packageInfo))

    assert.equal(actual, expected)
  },

}