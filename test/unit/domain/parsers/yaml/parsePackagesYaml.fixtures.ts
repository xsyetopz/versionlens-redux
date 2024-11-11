import {
  createDependencyRange,
  createPackageGitDescType,
  createPackageHostedDescType,
  createPackageNameDesc,
  createPackageParentDescType,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc,
  PackageDescriptor
} from '#domain/parsers';

export default {

  parsesDependencyEntries: {

    "test": `
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
  collection: "^1.1.0"
`,

    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(15, 15)),
        createPackageVersionDesc("1.2.3", createDependencyRange(24, 29)),
        createProjectVersionTypeDesc()
      ]),
      new PackageDescriptor([
        createPackageNameDesc("efts", createDependencyRange(376, 376)),
        createPackageVersionDesc("^2.0.4", createDependencyRange(382, 388)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("http", createDependencyRange(391, 391)),
        createPackageVersionDesc("*", createDependencyRange(397, 397), "", " "),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("transmogrify", createDependencyRange(421, 421)),
        createPackageVersionDesc("^0.4.0", createDependencyRange(448, 454), "", ""),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("test", createDependencyRange(489, 489)),
        createPackageVersionDesc(">=0.5.0 <0.12.0", createDependencyRange(496, 511)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("collection", createDependencyRange(515, 515)),
        createPackageVersionDesc("^1.1.0", createDependencyRange(528, 534)),
        createPackageParentDescType("dependencies")
      ]),
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
      new PackageDescriptor([
        createPackageNameDesc("pathify1", createDependencyRange(17, 17)),
        createPackagePathDescType("./some/test/path1", createDependencyRange(37, 54)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("pathify2", createDependencyRange(57, 57)),
        createPackagePathDescType("./some/test/path2", createDependencyRange(77, 94)),
        createPackageParentDescType("dependencies")
      ]),
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
      new PackageDescriptor([
        createPackageNameDesc("gitify1", createDependencyRange(17, 17)),
        createPackageGitDescType("git@github.com:munificent/kittens.git"),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("gitify2", createDependencyRange(76, 76)),
        createPackageGitDescType("git@github.com:munificent/dogs.git", "", "some-branch"),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("gitify3", createDependencyRange(167, 167)),
        createPackageGitDescType("git@github.com:munificent/birds.git", "path/to/birds"),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("gitify4", createDependencyRange(262, 262)),
        createPackageGitDescType("git@github.com:munificent/foxes.git"),
        createPackageParentDescType("dependencies")
      ]),
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
      new PackageDescriptor([
        createPackageNameDesc("hostify1", createDependencyRange(17, 17)),
        createPackageVersionDesc("1.0.0", createDependencyRange(40, 45)),
        createPackageHostedDescType("https://some-package-server.com"),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("hostify2", createDependencyRange(93, 93)),
        createPackageVersionDesc("2.0.0", createDependencyRange(116, 121), "", ""),
        createPackageHostedDescType("https://some-package-server.com"),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("hostify3", createDependencyRange(180, 180)),
        createPackageVersionDesc("3.0.0", createDependencyRange(203, 208)),
        createPackageHostedDescType("https://some-package-server.com", "testHostPackageAlias"),
        createPackageParentDescType("dependencies")
      ]),
    ]
  },

  parsesProjectVersionNoQuotes: {
    test: `version: 1.0.0`,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(0, 0)),
        createPackageVersionDesc("1.0.0", createDependencyRange(9, 14)),
        createProjectVersionTypeDesc()
      ])
    ]
  },

  parsesProjectVersionWithQuotes: {
    test: `version: '1.0.0'`,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(0, 0)),
        createPackageVersionDesc("1.0.0", createDependencyRange(10, 15)),
        createProjectVersionTypeDesc()
      ])
    ]
  },

  parsesProjectVersionWithComment: {
    test: `version: '1.0.0' # hello`,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(0, 0)),
        createPackageVersionDesc("1.0.0", createDependencyRange(10, 15), "", ""),
        createProjectVersionTypeDesc()
      ])
    ]
  },

  parsesEmptyProjectVersionWithComment: {
    test: `version: # hello`,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(0, 0)),
        createPackageVersionDesc("*", createDependencyRange(9, 9), "", " "),
        createProjectVersionTypeDesc()
      ])
    ]
  }

}