import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createPackageGitDescType,
  createPackageHostedDescType,
  createPackageNameDesc,
  createPackageParentDescType,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {

  parsesDependencyEntries: {

    test: `
name: newtify
version: 1.2.3
description: >-
  Have you been turned into a newt?  Would you like to be?
  This package can help. It has all of the
  newt-transmogrification functionality you have been looking
  for.
homepage: https://example-pet-store.com/newtify
documentation: https://example-pet-store.com/newtify/docs
environment:
  sdk: '>=2.0.0 <3.0.0'
dependencies:
  efts: ^2.0.4
  http: # blank with comments
  transmogrify:
    version: ^0.4.0 # complex version with comments
  test: '>=0.5.0 <0.12.0'
  collection: '^1.1.0'
`,

    expected: [
      new PackageDependency(
        createPackageResource('version', '1.2.3', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('version', createTextRange(15)),
          createPackageVersionDesc('1.2.3', createTextRange(24, 29)),
          createProjectVersionTypeDesc()
        ])
      ),
      new PackageDependency(
        createPackageResource('efts', '^2.0.4', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('efts', createTextRange(376)),
          createPackageVersionDesc('^2.0.4', createTextRange(382, 388)),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('http', '*', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('http', createTextRange(391)),
          createPackageVersionDesc('*', createTextRange(397, 397), '', ' '),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('transmogrify', '^0.4.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('transmogrify', createTextRange(421)),
          createPackageVersionDesc('^0.4.0', createTextRange(448, 454), '', ''),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('test', '>=0.5.0 <0.12.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('test', createTextRange(489)),
          createPackageVersionDesc('>=0.5.0 <0.12.0', createTextRange(496, 511)),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('collection', '^1.1.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('collection', createTextRange(515)),
          createPackageVersionDesc('^1.1.0', createTextRange(528, 534)),
          createPackageParentDescType('dependencies')
        ])
      ),
    ]
  },

  parsesPathDependencies: {

    test: `
dependencies:
  pathify1:
    path: ./some/test/path1
  pathify2:
    path: ./some/test/path2 # test comment
    `,
    expected: [
      new PackageDependency(
        createPackageResource('pathify1', './some/test/path1', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('pathify1', createTextRange(17)),
          createPackagePathDescType('./some/test/path1', createTextRange(37, 54)),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('pathify2', './some/test/path2', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('pathify2', createTextRange(57)),
          createPackagePathDescType('./some/test/path2', createTextRange(77, 94)),
          createPackageParentDescType('dependencies')
        ])
      ),
    ]
  },

  parsesGitDepencdencies: {
    test: `
dependencies:
  gitify1: 
    git: git@github.com:munificent/kittens.git
  gitify2: 
    git: 
      url: git@github.com:munificent/dogs.git
      ref: some-branch
  gitify3: 
    git: 
      url: git@github.com:munificent/birds.git
      path: path/to/birds
  gitify4: 
    git: git@github.com:munificent/foxes.git # test comment
    `  ,
    expected: [
      new PackageDependency(
        createPackageResource('gitify1', 'git@github.com:munificent/kittens.git', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('gitify1', createTextRange(17)),
          createPackageGitDescType('git@github.com:munificent/kittens.git'),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('gitify2', 'git@github.com:munificent/dogs.git', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('gitify2', createTextRange(76)),
          createPackageGitDescType('git@github.com:munificent/dogs.git', '', 'some-branch'),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('gitify3', 'git@github.com:munificent/birds.git', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('gitify3', createTextRange(167)),
          createPackageGitDescType('git@github.com:munificent/birds.git', 'path/to/birds'),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('gitify4', 'git@github.com:munificent/foxes.git', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('gitify4', createTextRange(262)),
          createPackageGitDescType('git@github.com:munificent/foxes.git'),
          createPackageParentDescType('dependencies')
        ])
      ),
    ]
  },

  parsesHostedDependencies: {

    test: `
dependencies:
  hostify1:
    version: 1.0.0
    hosted:  https://some-package-server.com
  hostify2:
    version: 2.0.0 # comments
    hosted:  https://some-package-server.com
  hostify3:
    version: 3.0.0
    hosted:
      name: testHostPackageAlias
      url: https://some-package-server.com
`,
    expected: [
      new PackageDependency(
        createPackageResource('hostify1', '1.0.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('hostify1', createTextRange(17)),
          createPackageVersionDesc('1.0.0', createTextRange(40, 45)),
          createPackageHostedDescType('https://some-package-server.com'),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('hostify2', '2.0.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('hostify2', createTextRange(93)),
          createPackageVersionDesc('2.0.0', createTextRange(116, 121), '', ''),
          createPackageHostedDescType('https://some-package-server.com'),
          createPackageParentDescType('dependencies')
        ])
      ),
      new PackageDependency(
        createPackageResource('hostify3', '3.0.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('hostify3', createTextRange(180)),
          createPackageVersionDesc('3.0.0', createTextRange(203, 208)),
          createPackageHostedDescType('https://some-package-server.com', 'testHostPackageAlias'),
          createPackageParentDescType('dependencies')
        ])
      ),
    ]
  },

  parsesProjectVersionNoQuotes: {
    test: `version: 1.0.0`,
    expected: [
      new PackageDependency(
        createPackageResource('version', '1.0.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('version', createTextRange(0)),
          createPackageVersionDesc('1.0.0', createTextRange(9, 14)),
          createProjectVersionTypeDesc()
        ])
      )
    ]
  },

  parsesProjectVersionWithQuotes: {
    test: `version: '1.0.0'`,
    expected: [
      new PackageDependency(
        createPackageResource('version', '1.0.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('version', createTextRange(0)),
          createPackageVersionDesc('1.0.0', createTextRange(10, 15)),
          createProjectVersionTypeDesc()
        ])
      )
    ]
  },

  parsesProjectVersionWithComment: {
    test: `version: '1.0.0' # hello`,
    expected: [
      new PackageDependency(
        createPackageResource('version', '1.0.0', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('version', createTextRange(0)),
          createPackageVersionDesc('1.0.0', createTextRange(10, 15), '', ''),
          createProjectVersionTypeDesc()
        ])
      )
    ]
  },

  parsesEmptyProjectVersionWithComment: {
    test: `version: # hello`,
    expected: [
      new PackageDependency(
        createPackageResource('version', '*', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('version', createTextRange(0)),
          createPackageVersionDesc('*', createTextRange(9), '', ' '),
          createProjectVersionTypeDesc()
        ])
      )
    ]
  },

  parsesAnyVersionKeyword: {
    test: `
    dependencies:
      dep1: any
    `,
    expected: [
      new PackageDependency(
        createPackageResource('dep1', '*', 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('dep1', createTextRange(25)),
          createPackageVersionDesc('*', createTextRange(31, 34)),
          createPackageParentDescType('dependencies')
        ])
      )
    ]
  }

}