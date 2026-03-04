import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createPackageGroupDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {

  case1: {
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
      new PackageDependency(
        createPackageResource('example.com/othermodule', 'v1.2.3', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('example.com/othermodule', createTextRange(63, 86)),
          createPackageVersionDesc('v1.2.3', createTextRange(87, 93), 'v'),
          createPackageGroupDesc('require', createTextRange(63, 93))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/spf13/cobra', 'v1.8.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/spf13/cobra', createTextRange(119, 141)),
          createPackageVersionDesc('v1.8.0', createTextRange(142, 148), 'v'),
          createPackageGroupDesc('require', createTextRange(119, 148))
        ])
      ),
      new PackageDependency(
        createPackageResource('gopkg.in/yaml.v3', 'v3.0.1', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('gopkg.in/yaml.v3', createTextRange(157, 173)),
          createPackageVersionDesc('v3.0.1', createTextRange(174, 180), 'v'),
          createPackageGroupDesc('require', createTextRange(157, 180))
        ])
      ),
      new PackageDependency(
        createPackageResource('k8s.io/klog/v2', 'v2.110.1', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('k8s.io/klog/v2', createTextRange(189, 203)),
          createPackageVersionDesc('v2.110.1', createTextRange(204, 212), 'v'),
          createPackageGroupDesc('require', createTextRange(189, 212))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/go-units', 'v0.5.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/go-units', createTextRange(501, 527)),
          createPackageVersionDesc('v0.5.0', createTextRange(528, 534), 'v'),
          createPackageGroupDesc('exclude', createTextRange(501, 534))
        ])
      ),
    ]
  },
  case2: {
    test: `module github.com/xxx/yyy

go 1.21

retract v1.1.0 // Published accidentally.

retract [v1.0.0, v1.0.5] // Build broken on some platforms.

require (
	github.com/docker/buildx v0.14.1
	github.com/docker/cli v26.1.3+incompatible
	github.com/docker/cli-docs-tool v0.7.0
	github.com/docker/docker v26.1.3+incompatible
	github.com/docker/go-connections v0.5.0
	github.com/docker/go-units v0.5.0
)

require golang.org/x/term v0.20.0

require (
	k8s.io/api v0.29.2 // indirect
	k8s.io/apimachinery v0.29.2 // indirect
	k8s.io/apiserver v0.29.2 // indirect
	k8s.io/client-go v0.29.2 // indirect
	k8s.io/klog/v2 v2.110.1 // indirect
	k8s.io/kube-openapi v0.0.0-20231010175941-2dd684a91f00 // indirect
	k8s.io/utils v0.0.0-20230726121419-3b25d923346b // indirect
)

exclude github.com/docker/go-units v0.5.0`,
    expected: [
      new PackageDependency(
        createPackageResource('github.com/docker/buildx', 'v0.14.1', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/buildx', createTextRange(146, 170)),
          createPackageVersionDesc('v0.14.1', createTextRange(171, 178), 'v'),
          createPackageGroupDesc('require', createTextRange(146, 178))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/cli', 'v26.1.3+incompatible', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/cli', createTextRange(180, 200)),
          createPackageVersionDesc('v26.1.3+incompatible', createTextRange(201, 223), 'v', '+incompatible'),
          createPackageGroupDesc('require', createTextRange(180, 223))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/cli-docs-tool', 'v0.7.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/cli-docs-tool', createTextRange(224, 253)),
          createPackageVersionDesc('v0.7.0', createTextRange(254, 261), 'v'),
          createPackageGroupDesc('require', createTextRange(224, 261))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/docker', 'v26.1.3+incompatible', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/docker', createTextRange(263, 286)),
          createPackageVersionDesc('v26.1.3+incompatible', createTextRange(287, 309), 'v', '+incompatible'),
          createPackageGroupDesc('require', createTextRange(263, 309))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/go-connections', 'v0.5.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/go-connections', createTextRange(310, 340)),
          createPackageVersionDesc('v0.5.0', createTextRange(341, 348), 'v'),
          createPackageGroupDesc('require', createTextRange(310, 348))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/go-units', 'v0.5.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/go-units', createTextRange(350, 374)),
          createPackageVersionDesc('v0.5.0', createTextRange(375, 382), 'v'),
          createPackageGroupDesc('require', createTextRange(350, 382))
        ])
      ),
      new PackageDependency(
        createPackageResource('golang.org/x/term', 'v0.20.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('golang.org/x/term', createTextRange(394, 412)),
          createPackageVersionDesc('v0.20.0', createTextRange(413, 420), 'v'),
          createPackageGroupDesc('require', createTextRange(394, 420))
        ])
      ),
      new PackageDependency(
        createPackageResource('k8s.io/api', 'v0.29.2', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('k8s.io/api', createTextRange(433, 443)),
          createPackageVersionDesc('v0.29.2', createTextRange(444, 451), 'v'),
          createPackageGroupDesc('require', createTextRange(433, 451))
        ])
      ),
      new PackageDependency(
        createPackageResource('k8s.io/apimachinery', 'v0.29.2', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('k8s.io/apimachinery', createTextRange(465, 484)),
          createPackageVersionDesc('v0.29.2', createTextRange(485, 492), 'v'),
          createPackageGroupDesc('require', createTextRange(465, 492))
        ])
      ),
      new PackageDependency(
        createPackageResource('k8s.io/apiserver', 'v0.29.2', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('k8s.io/apiserver', createTextRange(507, 523)),
          createPackageVersionDesc('v0.29.2', createTextRange(524, 531), 'v'),
          createPackageGroupDesc('require', createTextRange(507, 531))
        ])
      ),
      new PackageDependency(
        createPackageResource('k8s.io/client-go', 'v0.29.2', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('k8s.io/client-go', createTextRange(546, 562)),
          createPackageVersionDesc('v0.29.2', createTextRange(563, 570), 'v'),
          createPackageGroupDesc('require', createTextRange(546, 570))
        ])
      ),
      new PackageDependency(
        createPackageResource('k8s.io/klog/v2', 'v2.110.1', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('k8s.io/klog/v2', createTextRange(585, 599)),
          createPackageVersionDesc('v2.110.1', createTextRange(600, 608), 'v'),
          createPackageGroupDesc('require', createTextRange(585, 608))
        ])
      ),
      new PackageDependency(
        createPackageResource('github.com/docker/go-units', 'v0.5.0', 'test/path/go.mod'),
        new PackageDescriptor([
          createPackageNameDesc('github.com/docker/go-units', createTextRange(753, 777)),
          createPackageVersionDesc('v0.5.0', createTextRange(778, 785), 'v'),
          createPackageGroupDesc('exclude', createTextRange(753, 785))
        ])
      ),
    ]
  }

}
