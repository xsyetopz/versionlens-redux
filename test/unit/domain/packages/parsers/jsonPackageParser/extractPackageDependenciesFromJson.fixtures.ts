import {
  PackageDescriptor,
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageNameDescriptor,
  TPackageParentDescriptor,
  TPackagePathDescriptor,
  TPackageTypeDescriptor,
  TPackageVersionDescriptor
} from "domain/packages";
import { KeyDictionary } from 'domain/utils';

export default {

  extractDependencyEntries: {

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
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "Package1",
            nameRange: {
              start: 17,
              end: 17
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "1.0.0",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 29,
              end: 34
            },
          },
          parent: <TPackageParentDescriptor>{
            type: PackageDescriptorType.parent,
            path: "dependencies"
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "Package2",
            nameRange: {
              start: 36,
              end: 36
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "github:repo/project#semver:1.2.3",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 48,
              end: 80
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "dependencies"
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "Package3",
            nameRange: {
              start: 82,
              end: 82
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "*",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 94,
              end: 95
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "dependencies"
          }
        },
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "ComplexPackage1",
            nameRange: {
              start: 97,
              end: 97
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "1.2.3",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 127,
              end: 132
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "dependencies"
          }
        },
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "NameOverrides@1",
            nameRange: {
              end: 135,
              start: 135
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "1.0.0",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              end: 159,
              start: 154
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "dependencies"
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "PathPackage1",
            nameRange: {
              start: 161,
              end: 161
            },
          },
          path: <TPackagePathDescriptor>{
            type: "path",
            path: "some/path/project",
            pathRange: {
              start: 185,
              end: 202
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "dependencies"
          }
        },
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "GitPackage1",
            nameRange: {
              start: 205,
              end: 205
            },
          },
          git: <TPackageGitDescriptor>{
            type: "git",
            gitUrl: "git@github.com:munificent/kittens.git",
            gitRef: "",
            gitPath: ""
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "dependencies"
          }
        },
      },
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
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "childPackage1",
            nameRange: {
              start: 32,
              end: 32
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "2.0.0",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 49,
              end: 54
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "overrides"
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "childPackage2",
            nameRange: {
              start: 56,
              end: 56
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "3.0.0",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 73,
              end: 78
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "overrides"
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 3,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "childPackage3",
            nameRange: {
              start: 99,
              end: 99
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "4.0.0",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 116,
              end: 121
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "overrides"
          }
        }
      },
    ]
  }
}