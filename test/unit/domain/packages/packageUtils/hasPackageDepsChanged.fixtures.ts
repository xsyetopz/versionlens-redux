import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createIgnoreChangesDesc,
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {
  single: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(0, 1)),
        createPackageVersionDesc('1.0.0', createTextRange(2, 3)),
      ])
    )
  ],
  singleWithDiffVersion: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.1.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(0, 1)),
        createPackageVersionDesc('1.1.0', createTextRange(2, 3)),
      ])
    )
  ],
  singleWithDiffNameRange: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(4, 5)),
        createPackageVersionDesc('1.0.0', createTextRange(2, 3)),
      ])
    )
  ],
  singleWithDiffVersionRange: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(0, 1)),
        createPackageVersionDesc('1.0.0', createTextRange(4, 5)),
      ])
    )
  ],
  multiple: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(4, 5)),
        createPackageVersionDesc('1.0.0', createTextRange(6, 7)),
      ])
    ),
    new PackageDependency(
      createPackageResource("testPackage2", "2.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage2', createTextRange(8, 9)),
        createPackageVersionDesc('2.0.0', createTextRange(10, 11)),
      ])
    )
  ],
  multipleWithDiffVersion: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(4, 5)),
        createPackageVersionDesc('1.0.0', createTextRange(6, 7)),
      ])
    ),
    new PackageDependency(
      createPackageResource("testPackage2", "2.1.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage2', createTextRange(8, 9)),
        createPackageVersionDesc('2.1.0', createTextRange(10, 11)),
      ])
    )
  ],
  multipleWithDiffNameRange: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(4, 5)),
        createPackageVersionDesc('1.0.0', createTextRange(6, 7)),
      ])
    ),
    new PackageDependency(
      createPackageResource("testPackage2", "2.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage2', createTextRange(12, 13)),
        createPackageVersionDesc('2.0.0', createTextRange(10, 11)),
      ])
    )
  ],
  multipleWithDiffVersionRange: [
    new PackageDependency(
      createPackageResource("testPackage1", "1.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(4, 5)),
        createPackageVersionDesc('1.0.0', createTextRange(12, 13)),
      ])
    ),
    new PackageDependency(
      createPackageResource("testPackage2", "2.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage2', createTextRange(8, 9)),
        createPackageVersionDesc('2.0.0', createTextRange(10, 11)),
      ])
    )
  ],
  ignoresChanges: [
    new PackageDependency(
      createPackageResource("testPackage1", "10.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage1', createTextRange(4, 5)),
        createPackageVersionDesc('10.0.0', createTextRange(12, 13)),
        createIgnoreChangesDesc()
      ])
    ),
    new PackageDependency(
      createPackageResource("testPackage2", "20.0.0", "test/path"),
      new PackageDescriptor([
        createPackageNameDesc('testPackage2', createTextRange(8, 9)),
        createPackageVersionDesc('20.0.0', createTextRange(10, 11)),
        createIgnoreChangesDesc()
      ])
    )
  ],
}