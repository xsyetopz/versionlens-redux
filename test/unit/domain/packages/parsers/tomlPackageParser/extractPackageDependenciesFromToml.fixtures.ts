import {
  PackageDescriptor,
  createDependencyRange,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from "domain/packages";

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
          createDependencyRange(28, 28)
        ),
        createPackageVersionDesc("1.0.97", createDependencyRange(37, 43))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "indexmap",
          createDependencyRange(51, 51)
        ),
        createPackageVersionDesc("1.0", createDependencyRange(75, 78))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "awesome",
          createDependencyRange(120, 120)
        ),
        createPackageVersionDesc("1.3.5", createDependencyRange(146, 151))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "serde_derive",
          createDependencyRange(185, 185)
        ),
        createPackageVersionDesc("1.0", createDependencyRange(201, 204))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "serde_json",
          createDependencyRange(212, 212)
        ),
        createPackageVersionDesc("1.0", createDependencyRange(226, 229))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "smallvec",
          createDependencyRange(237, 237)
        ),
        createPackageGitDescType("https://github.com/servo/rust-smallvec.git")
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "bitflags",
          createDependencyRange(309, 309)
        ),
        createPackagePathDescType("my-bitflags", createDependencyRange(330, 341))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "serde",
          createDependencyRange(395, 395)
        ),
        createPackageVersionDesc("1.0.97", createDependencyRange(404, 410))
      ]),
    ]
  },

  parsesProjectVersionEntries: {
    test: `
      [package]
      version = "1.0.97"

      [project]
      version = "1.3.5"
    `,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(23, 23)),
        createPackageVersionDesc("1.0.97", createDependencyRange(34, 40)),
        createProjectVersionTypeDesc()
      ]),
      new PackageDescriptor([
        createPackageNameDesc("version", createDependencyRange(65, 65)),
        createPackageVersionDesc("1.3.5", createDependencyRange(76, 81)),
        createProjectVersionTypeDesc()
      ]),
    ]
  }

}