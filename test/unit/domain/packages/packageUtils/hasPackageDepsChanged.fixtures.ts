import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createIgnoreChangesDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {
  single: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(0, 1),
      // versionRange
      createTextRange(2, 3),
      new PackageDescriptor([])
    )
  ],
  singleWithDiffVersion: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.1.0",
        "test/path"
      ),
      //nameRange
      createTextRange(0, 1),
      // versionRange
      createTextRange(2, 3),
      new PackageDescriptor([])
    )
  ],
  singleWithDiffNameRange: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(4, 5),
      // versionRange
      createTextRange(2, 3),
      new PackageDescriptor([])
    )
  ],
  singleWithDiffVersionRange: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(0, 1),
      // versionRange
      createTextRange(4, 5),
      new PackageDescriptor([])
    )
  ],
  multiple: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(4, 5),
      // versionRange
      createTextRange(6, 7),
      new PackageDescriptor([])
    ),
    new PackageDependency(
      createPackageResource(
        "testPackage2",
        "2.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(8, 9),
      // versionRange
      createTextRange(10, 11),
      new PackageDescriptor([])
    )
  ],
  multipleWithDiffVersion: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(4, 5),
      // versionRange
      createTextRange(6, 7),
      new PackageDescriptor([])
    ),
    new PackageDependency(
      createPackageResource(
        "testPackage2",
        "2.1.0",
        "test/path"
      ),
      //nameRange
      createTextRange(8, 9),
      // versionRange
      createTextRange(10, 11),
      new PackageDescriptor([])
    )
  ],
  multipleWithDiffNameRange: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(4, 5),
      // versionRange
      createTextRange(6, 7),
      new PackageDescriptor([])
    ),
    new PackageDependency(
      createPackageResource(
        "testPackage2",
        "2.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(12, 13),
      // versionRange
      createTextRange(10, 11),
      new PackageDescriptor([])
    )
  ],
  multipleWithDiffVersionRange: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "1.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(4, 5),
      // versionRange
      createTextRange(12, 13),
      new PackageDescriptor([])
    ),
    new PackageDependency(
      createPackageResource(
        "testPackage2",
        "2.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(8, 9),
      // versionRange
      createTextRange(10, 11),
      new PackageDescriptor([])
    )
  ],
  ignoresChanges: [
    new PackageDependency(
      createPackageResource(
        "testPackage1",
        "10.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(4, 5),
      // versionRange
      createTextRange(12, 13),
      new PackageDescriptor([
        createIgnoreChangesDesc()
      ])
    ),
    new PackageDependency(
      createPackageResource(
        "testPackage2",
        "20.0.0",
        "test/path"
      ),
      //nameRange
      createTextRange(8, 9),
      // versionRange
      createTextRange(10, 11),
      new PackageDescriptor([
        createIgnoreChangesDesc()
      ])
    )
  ],
}