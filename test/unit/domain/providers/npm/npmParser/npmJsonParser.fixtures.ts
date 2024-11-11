import {
  PackageDescriptor,
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageParentDescriptor,
  TPackageTypeDescriptor,
  TPackageVersionDescriptor
} from '#domain/parsers';
import { KeyDictionary } from '#domain/utils';

export default {

  matchesPackageManagerExpressions: {
    test: {
      packageManager: "pnpm@9.1.2",
    },
    expected: [
      <PackageDescriptor>{
        typeCount: 4,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          ignoreChanges: {
            type: 'ignoreChanges'
          },
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "pnpm@9.1.2",
            nameRange: {
              start: 18,
              end: 18
            },
          },
          version: <TPackageVersionDescriptor>{
            type: "version",
            version: "9.1.2",
            versionAppend: "",
            versionPrepend: "",
            versionRange: {
              start: 24,
              end: 29
            },
          },
          parent: <TPackageParentDescriptor>{
            type: "parent",
            path: "packageManager"
          }
        }
      }
    ]
  },

  matchesPackageManagerShaExpressions: {
    test: {
      packageManager:
        'pnpm@9.1.2+sha512.14e915759c11f77eac07faba4d019c193ec8637229e62ec99eefb7cf3c3b75c64447882b7c485142451ee3a6b408059cdfb7b7fa0341b975f12d0f7629c71195',
    },
    expected: [
      <PackageDescriptor>{
        typeCount: 4,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          ignoreChanges: {
            type: 'ignoreChanges',
          },
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: 'pnpm@9.1.2+sha512.14e915759c11f77eac07faba4d019c193ec8637229e62ec99eefb7cf3c3b75c64447882b7c485142451ee3a6b408059cdfb7b7fa0341b975f12d0f7629c71195',
            nameRange: {
              start: 18,
              end: 18,
            },
          },
          version: <TPackageVersionDescriptor>{
            type: 'version',
            version:
              '9.1.2+sha512.14e915759c11f77eac07faba4d019c193ec8637229e62ec99eefb7cf3c3b75c64447882b7c485142451ee3a6b408059cdfb7b7fa0341b975f12d0f7629c71195',
            versionAppend: '',
            versionPrepend: '',
            versionRange: {
              start: 24,
              end: 165,
            },
          },
          parent: <TPackageParentDescriptor>{
            type: 'parent',
            path: 'packageManager',
          },
        },
      },
    ],
  },

  matchesPackageManagerShaPrereleaseExpressions: {
    test: {
      packageManager:
        'pnpm@9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac',
    },
    expected: [
      <PackageDescriptor>{
        typeCount: 4,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          ignoreChanges: {
            type: 'ignoreChanges',
          },
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: 'pnpm@9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac',
            nameRange: {
              start: 18,
              end: 18,
            },
          },
          version: <TPackageVersionDescriptor>{
            type: 'version',
            version:
              '9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac',
            versionAppend: '',
            versionPrepend: '',
            versionRange: {
              start: 24,
              end: 95,
            },
          },
          parent: <TPackageParentDescriptor>{
            type: 'parent',
            path: 'packageManager',
          },
        },
      },
    ],
  },
  
};