import {
  PackageDescriptor,
  createPackageGroupDesc,
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange
} from '#domain/parsers';

const smokeTestText = `module github.com/xxx/yyy

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

exclude github.com/docker/go-units v0.5.0`;

const getSmokeExpected = () => {
  const text = smokeTestText;

  const getDesc = (name: string, version: string, group: string, useLastIndexOf: boolean = false) => {
    const nameStart = useLastIndexOf ? text.lastIndexOf(name) : text.indexOf(name);
    const versionStart = text.indexOf(version, nameStart + name.length);
    const versionEnd = versionStart + version.length;

    const descriptors = [];
    if (name) {
      descriptors.push(createPackageNameDesc(name, createTextRange(nameStart, nameStart + name.length)));
    }
    descriptors.push(
      createPackageVersionDesc(
        version,
        createTextRange(versionStart, versionEnd),
        'v',
        version.endsWith('+incompatible') ? '+incompatible' : ''
      )
    );
    descriptors.push(createPackageGroupDesc(group, createTextRange(name ? nameStart : versionStart, versionEnd)));

    return new PackageDescriptor(descriptors);
  };

  return [
    getDesc('github.com/docker/buildx', 'v0.14.1', 'require'),
    getDesc('github.com/docker/cli', 'v26.1.3+incompatible', 'require'),
    getDesc('github.com/docker/cli-docs-tool', 'v0.7.0', 'require'),
    getDesc('github.com/docker/docker', 'v26.1.3+incompatible', 'require'),
    getDesc('github.com/docker/go-connections', 'v0.5.0', 'require'),
    getDesc('github.com/docker/go-units', 'v0.5.0', 'require'),
    getDesc('golang.org/x/term', 'v0.20.0', 'require'),
    getDesc('k8s.io/api', 'v0.29.2', 'require'),
    getDesc('k8s.io/apimachinery', 'v0.29.2', 'require'),
    getDesc('k8s.io/apiserver', 'v0.29.2', 'require'),
    getDesc('k8s.io/client-go', 'v0.29.2', 'require'),
    getDesc('k8s.io/klog/v2', 'v2.110.1', 'require'),
    getDesc('github.com/docker/go-units', 'v0.5.0', 'exclude', true)
  ];
};

export default {

  parsesGoMod: {
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
      new PackageDescriptor([
        createPackageNameDesc('example.com/othermodule', createTextRange(63, 86)),
        createPackageVersionDesc('v1.2.3', createTextRange(87, 93), 'v'),
        createPackageGroupDesc('require', createTextRange(63, 93))
      ]),
      new PackageDescriptor([
        createPackageNameDesc('github.com/spf13/cobra', createTextRange(119, 141)),
        createPackageVersionDesc('v1.8.0', createTextRange(142, 148), 'v'),
        createPackageGroupDesc('require', createTextRange(119, 148))
      ]),
      new PackageDescriptor([
        createPackageNameDesc('gopkg.in/yaml.v3', createTextRange(157, 173)),
        createPackageVersionDesc('v3.0.1', createTextRange(174, 180), 'v'),
        createPackageGroupDesc('require', createTextRange(157, 180))
      ]),
      new PackageDescriptor([
        createPackageNameDesc('k8s.io/klog/v2', createTextRange(189, 203)),
        createPackageVersionDesc('v2.110.1', createTextRange(204, 212), 'v'),
        createPackageGroupDesc('require', createTextRange(189, 212))
      ]),
      new PackageDescriptor([
        createPackageNameDesc('github.com/docker/go-units', createTextRange(501, 527)),
        createPackageVersionDesc('v0.5.0', createTextRange(528, 534), 'v'),
        createPackageGroupDesc('exclude', createTextRange(501, 534))
      ]),
    ]
  },

  smoke: {
    test: smokeTestText,
    expected: getSmokeExpected()
  }

}
