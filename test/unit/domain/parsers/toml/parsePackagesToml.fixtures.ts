import {
  PackageDescriptor,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc,
  createTextRange
} from '#domain/parsers';

export default {

  parsesDependencyEntries: {
    test: `
      [dependencies]
      serde = "1.0.97"
      indexmap = { version = "1.0", optional = true }

      [dependencies.awesome]
      version = "1.3.5"

      [dev-dependencies]
      serde_derive = "1.0"
      serde_json = "1.0"
      smallvec = { git = "https://github.com/servo/rust-smallvec.git" }
      bitflags = { path = "my-bitflags" }

      [tool.poetry.group.dev.dependencies]
      serde = "1.0.97"
    `,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc(
          "serde",
          createTextRange(28, 28)
        ),
        createPackageVersionDesc("1.0.97", createTextRange(37, 43))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "indexmap",
          createTextRange(51, 51)
        ),
        createPackageVersionDesc("1.0", createTextRange(75, 78))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "awesome",
          createTextRange(120, 120)
        ),
        createPackageVersionDesc("1.3.5", createTextRange(146, 151))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "serde_derive",
          createTextRange(185, 185)
        ),
        createPackageVersionDesc("1.0", createTextRange(201, 204))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "serde_json",
          createTextRange(212, 212)
        ),
        createPackageVersionDesc("1.0", createTextRange(226, 229))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "smallvec",
          createTextRange(237, 237)
        ),
        createPackageGitDescType("https://github.com/servo/rust-smallvec.git")
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "bitflags",
          createTextRange(309, 309)
        ),
        createPackagePathDescType("my-bitflags", createTextRange(330, 341))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "serde",
          createTextRange(395, 395)
        ),
        createPackageVersionDesc("1.0.97", createTextRange(404, 410))
      ]),
    ]
  },

  parsesPackageVersionEntries: {
    test: `
      [package]
      version = "1.0.97"
      description = "should ignore this field"

      [dev-dependencies]
      backtrace = "1.3.5"
    `,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createTextRange(23, 23)),
        createPackageVersionDesc("1.0.97", createTextRange(34, 40)),
        createProjectVersionTypeDesc()
      ]),
      new PackageDescriptor([
        createPackageNameDesc("backtrace", createTextRange(121, 121)),
        createPackageVersionDesc("1.3.5", createTextRange(134, 139))
      ]),
    ]
  },

  parsesProjectVersionEntries: {
    test: `
      [project]
      version = "1.0.97"
      description = "should ignore this field"

      [dev-dependencies]
      backtrace = "1.3.5"
    `,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createTextRange(23, 23)),
        createPackageVersionDesc("1.0.97", createTextRange(34, 40)),
        createProjectVersionTypeDesc()
      ]),
      new PackageDescriptor([
        createPackageNameDesc("backtrace", createTextRange(121, 121)),
        createPackageVersionDesc("1.3.5", createTextRange(134, 139))
      ]),
    ]
  }

}