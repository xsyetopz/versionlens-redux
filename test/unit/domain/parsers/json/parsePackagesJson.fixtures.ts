import {
  PackageDescriptor,
  createDependencyRange,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackageParentDescType,
  createPackagePathDescType,
  createPackageVersionDesc
} from '#domain/parsers';

export default {

  parsesDependencyEntries: {

    test: {
      "dependencies": {
        "Package1": "1.0.0",
        "Package2": "github:repo/project#semver:1.2.3",
        "Package3": "*",
        "ComplexPackage1": {
          "version": "1.2.3"
        },
        "NameOverrides@1": "1.0.0",
        "PathPackage1": {
          "path": "some/path/project"
        },
        "GitPackage1": {
          "repository": "git@github.com:munificent/kittens.git"
        },
      },
      "scripts": {
        "script1": "run me",
      }
    },

    expected: [
      new PackageDescriptor([
        createPackageNameDesc("Package1", createDependencyRange(17, 17)),
        createPackageVersionDesc("1.0.0", createDependencyRange(29, 34)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("Package2", createDependencyRange(36, 36)),
        createPackageVersionDesc("github:repo/project#semver:1.2.3", createDependencyRange(48, 80)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("Package3", createDependencyRange(82, 82)),
        createPackageVersionDesc("*", createDependencyRange(94, 95)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("ComplexPackage1", createDependencyRange(97, 97)),
        createPackageVersionDesc("1.2.3", createDependencyRange(127, 132)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("NameOverrides@1", createDependencyRange(135, 135)),
        createPackageVersionDesc("1.0.0", createDependencyRange(154, 159)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("PathPackage1", createDependencyRange(161, 161)),
        createPackagePathDescType("some/path/project", createDependencyRange(185, 202)),
        createPackageParentDescType("dependencies")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("GitPackage1", createDependencyRange(205, 205)),
        createPackageGitDescType("git@github.com:munificent/kittens.git"),
        createPackageParentDescType("dependencies")
      ]),
    ]
  },

  matchesPathExpressions: {
    test: {
      "overrides": {
        "parentPackage1": {
          "childPackage1": "2.0.0",
          "childPackage2": "3.0.0",
        },
        "parentPackage2": {
          "childPackage3": "4.0.0",
        }
      }
    },
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("childPackage1", createDependencyRange(32, 32)),
        createPackageVersionDesc("2.0.0", createDependencyRange(49, 54)),
        createPackageParentDescType("overrides")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("childPackage2", createDependencyRange(56, 56)),
        createPackageVersionDesc("3.0.0", createDependencyRange(73, 78)),
        createPackageParentDescType("overrides")
      ]),
      new PackageDescriptor([
        createPackageNameDesc("childPackage3", createDependencyRange(99, 99)),
        createPackageVersionDesc("4.0.0", createDependencyRange(116, 121)),
        createPackageParentDescType("overrides")
      ]),
    ]
  },
};
