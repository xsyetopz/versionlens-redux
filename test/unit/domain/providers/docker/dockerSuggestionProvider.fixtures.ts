import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageParentDescType,
  createPackagePathDescType,
  createPackageRegistryDescType,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {
  dockerfile: {
    test: `
    FROM image/test1:1.0.0
    FROM image/test2:2.0.0 # test comments
    FROM image/test3:3.0.0 as AliasName
    FROM image/test4:4.0.0 as AliasNameWithComments # test comments
    FROM image/test1
    FROM image/test2 # test comments
    FROM image/test3 as AliasName
    FROM image/test4 as AliasNameWithComments # test comments
    FROM
    `,
    expected: <PackageDependency[]>[
      new PackageDependency(
        createPackageResource('image/test1', '1.0.0', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test1', createTextRange(10, 21)),
          createPackageVersionDesc('1.0.0', createTextRange(22, 27)),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test2', '2.0.0', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test2', createTextRange(37, 48)),
          createPackageVersionDesc('2.0.0', createTextRange(49, 54)),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test3', '3.0.0', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test3', createTextRange(80, 91)),
          createPackageVersionDesc('3.0.0', createTextRange(92, 97)),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test4', '4.0.0', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test4', createTextRange(120, 131)),
          createPackageVersionDesc('4.0.0', createTextRange(132, 137)),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test1', '', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test1', createTextRange(188, 199)),
          createPackageVersionDesc('', createTextRange(199, 199), ':'),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test2', '', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test2', createTextRange(209, 220)),
          createPackageVersionDesc('', createTextRange(220, 220), ':'),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test3', '', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test3', createTextRange(246, 257)),
          createPackageVersionDesc('', createTextRange(257, 257), ':'),
        ])
      ),
      new PackageDependency(
        createPackageResource('image/test4', '', 'test/path/dockerfile'),
        new PackageDescriptor([
          createPackageNameDesc('image/test4', createTextRange(280, 291)),
          createPackageVersionDesc('', createTextRange(291, 291), ':'),
        ])
      )
    ]
  },
  compose: {
    test: `
      services:
        web:
          image: node:alpine-22
        api:
          image: 'alpine:3'
        data:
          image: postrgres
        custom-build:
          build: .
        custom-context-default:
          build:
            context: .
        custom-context-dockerfile:
          build:
            context: .
            dockerfile: custom.dockerfile
        image-number:
          image: 123456
        image-registry:
          image: mcr.microsoft.com/dotnet/sdk:7.0
        empty-image:
          image:
        empty-build:
          build:
    `,
    expected: <PackageDependency[]>[
      new PackageDependency(
        createPackageResource('node', 'alpine-22', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('node', createTextRange(47, 51)),
          createPackageVersionDesc('alpine-22', createTextRange(52, 61)),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('alpine', '3', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('alpine', createTextRange(93, 99)),
          createPackageVersionDesc('3', createTextRange(100, 101)),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('postrgres', '', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('postrgres', createTextRange(134, 143)),
          createPackageVersionDesc('', createTextRange(143, 143), ':'),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('./dockerfile', './dockerfile', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('./dockerfile', createTextRange(183, 184)),
          createPackagePathDescType('./dockerfile', createTextRange(183, 184)),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('./dockerfile', './dockerfile', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('./dockerfile', createTextRange(255, 256)),
          createPackagePathDescType('./dockerfile', createTextRange(255, 256)),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('./custom.dockerfile', './custom.dockerfile', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('./custom.dockerfile', createTextRange(330, 331)),
          createPackagePathDescType('./custom.dockerfile', createTextRange(330, 331)),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('123456', '', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('123456', createTextRange(413, 419)),
          createPackageVersionDesc('', createTextRange(419), ':'),
          createPackageParentDescType('services.*')
        ])
      ),
      new PackageDependency(
        createPackageResource('dotnet/sdk', '7.0', 'test/path/compose.yaml'),
        new PackageDescriptor([
          createPackageNameDesc('dotnet/sdk', createTextRange(479, 489)),
          createPackageVersionDesc('7.0', createTextRange(490, 493)),
          createPackageParentDescType('services.*'),
          createPackageRegistryDescType('mcr.microsoft.com')
        ])
      ),
    ]
  }
}