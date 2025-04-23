import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {
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
  expected: <PackageDependency[]>[
    new PackageDependency(
      createPackageResource('example.com/othermodule', 'v1.2.3', 'test/path/go.mod'),
      new PackageDescriptor([
        createPackageNameDesc('example.com/othermodule', createTextRange(63)),
        createPackageVersionDesc('v1.2.3', createTextRange(87, 93), 'v'),
      ])
    ),
    new PackageDependency(
      createPackageResource('github.com/spf13/cobra', 'v1.8.0', 'test/path/go.mod'),
      new PackageDescriptor([
        createPackageNameDesc('github.com/spf13/cobra', createTextRange(119)),
        createPackageVersionDesc('v1.8.0', createTextRange(142, 148), 'v'),
      ])
    ),
    new PackageDependency(
      createPackageResource('gopkg.in/yaml.v3', 'v3.0.1', 'test/path/go.mod'),
      new PackageDescriptor([
        createPackageNameDesc('gopkg.in/yaml.v3', createTextRange(157)),
        createPackageVersionDesc('v3.0.1', createTextRange(174, 180), 'v'),
      ])
    ),
    new PackageDependency(
      createPackageResource('k8s.io/klog/v2', 'v2.110.1', 'test/path/go.mod'),
      new PackageDescriptor([
        createPackageNameDesc('k8s.io/klog/v2', createTextRange(189)),
        createPackageVersionDesc('v2.110.1', createTextRange(204, 212), 'v'),
      ])
    ),
    new PackageDependency(
      createPackageResource('github.com/docker/go-units', 'v0.5.0', 'test/path/go.mod'),
      new PackageDescriptor([
        createPackageNameDesc('github.com/docker/go-units', createTextRange(501)),
        createPackageVersionDesc('v0.5.0', createTextRange(528, 534), 'v'),
      ])
    ),
  ]

}