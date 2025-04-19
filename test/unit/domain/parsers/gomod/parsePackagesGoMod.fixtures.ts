import {
  PackageDescriptor,
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageTypeDescriptor,
  TPackageVersionDescriptor
} from '#domain/parsers';
import { KeyDictionary } from '#domain/utils';

export default {

  extractNoDependencies: {
    test: `
      module github.com/xxx/yyy

      go 1.20
    `,
    expected: []
  },

  extractDependencies: {
    test: `
      module github.com/xxx/yyy

      go 1.20

      require example.com/othermodule v1.2.3

      require (
        github.com/spf13/cobra v1.8.0
        gopkg.in/yaml.v3 v3.0.1
        k8s.io/klog/v2 v2.110.1 // test comment
      )

      // should ignore pseudo versions
      k8s.io/utils v0.0.0-20230726121419-3b25d923346b

      // should ignore retract versions
      retract v1.1.0 // Published accidentally.
      retract [v1.0.0, v1.0.5] // Build broken on some platforms.

      exclude github.com/docker/go-units v0.5.0
    `,
    expected: [
      <PackageDescriptor>{
        typeCount: 2,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "example.com/othermodule",
            nameRange: {
              start: 63,
              end: 63
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "v1.2.3",
            versionAppend: "",
            versionPrepend: "v",
            versionRange: {
              start: 87,
              end: 93
            },
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 2,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "github.com/spf13/cobra",
            nameRange: {
              start: 119,
              end: 119
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "v1.8.0",
            versionAppend: "",
            versionPrepend: "v",
            versionRange: {
              start: 142,
              end: 148
            },
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 2,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "gopkg.in/yaml.v3",
            nameRange: {
              start: 157,
              end: 157
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "v3.0.1",
            versionAppend: "",
            versionPrepend: "v",
            versionRange: {
              start: 174,
              end: 180
            },
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 2,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "k8s.io/klog/v2",
            nameRange: {
              start: 189,
              end: 189
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "v2.110.1",
            versionAppend: "",
            versionPrepend: "v",
            versionRange: {
              start: 204,
              end: 212
            },
          }
        }
      },
      <PackageDescriptor>{
        typeCount: 2,
        types: <KeyDictionary<TPackageTypeDescriptor>>{
          name: <TPackageNameDescriptor>{
            type: PackageDescriptorType.name,
            name: "github.com/docker/go-units",
            nameRange: {
              start: 501,
              end: 501
            },
          },
          version: <TPackageVersionDescriptor>{
            type: PackageDescriptorType.version,
            version: "v0.5.0",
            versionAppend: "",
            versionPrepend: "v",
            versionRange: {
              start: 528,
              end: 534
            },
          }
        }
      },
    ]
  },
}