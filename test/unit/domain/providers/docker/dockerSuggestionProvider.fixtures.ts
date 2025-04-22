import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {
  test: `
    FROM image/test1:1.0.0
    FROM image/test2:2.0.0 # test comments
    FROM image/test3:3.0.0 as AliasName
    FROM image/test4:4.0.0 as AliasNameWithComments # test comments
  `,
  expected: <PackageDependency[]>[
    new PackageDependency(
      createPackageResource('image/test1', '1.0.0', 'test/path'),
      new PackageDescriptor([
        createPackageNameDesc('image/test1', createTextRange(10, 21)),
        createPackageVersionDesc('1.0.0', createTextRange(22, 27)),
      ])
    ),
    new PackageDependency(
      createPackageResource('image/test2', '2.0.0', 'test/path'),
      new PackageDescriptor([
        createPackageNameDesc('image/test2', createTextRange(37, 48)),
        createPackageVersionDesc('2.0.0', createTextRange(49, 54)),
      ])
    ),
    new PackageDependency(
      createPackageResource('image/test3', '3.0.0', 'test/path'),
      new PackageDescriptor([
        createPackageNameDesc('image/test3', createTextRange(80, 91)),
        createPackageVersionDesc('3.0.0', createTextRange(92, 97)),
      ])
    ),
    new PackageDependency(
      createPackageResource('image/test4', '4.0.0', 'test/path'),
      new PackageDescriptor([
        createPackageNameDesc('image/test4', createTextRange(120, 131)),
        createPackageVersionDesc('4.0.0', createTextRange(132, 137)),
      ])
    )
  ]
}