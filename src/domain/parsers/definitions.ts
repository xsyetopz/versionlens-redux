/**
 * Enum representing the different types of package descriptors.
 */
export enum PackageDescriptorType {
  /** The name of the package. */
  name = "name",
  /** The version of the package. */
  version = "version",
  /** A local path to the package. */
  path = "path",
  /** A Git URL for the package. */
  git = "git",
  /** A GitHub shorthand for the package. */
  github = "github",
  /** A hosted registry URL for the package. */
  hosted = "hosted",
  /** Indicates that changes should be ignored. */
  ignoreChanges = "ignoreChanges",
  /** The version of the project itself. */
  projectVersion = "projectVersion",
  /** A Docker image descriptor. */
  image = "image",
  /** A Docker build descriptor. */
  build = 'build',
  /** A package registry descriptor. */
  registry = 'registry',
  /** The group descriptor for the full package entry. */
  group = 'group'
}

/**
 * Represents a range of text within a file.
 */
export type PackageTextRange = {
  /** The starting character offset. */
  start: number;
  /** The ending character offset. */
  end: number;
};

/**
 * Base type for package descriptors.
 */
export type PackageType = {
  /** The type of the descriptor. */
  type: keyof typeof PackageDescriptorType
}

/**
 * Descriptor for a package name.
 */
export type PackageNameDescriptor = PackageType & {
  /** The name of the package. */
  name: string
  /** The range of the name in the file. */
  nameRange: PackageTextRange
}

/**
 * Descriptor for a package version.
 */
export type PackageVersionDescriptor = PackageType & {
  /** The version string. */
  version: string
  /** The range of the version in the file. */
  versionRange: PackageTextRange
  /** Text to prepend when updating the version. */
  versionPrepend: string
  /** Text to append when updating the version. */
  versionAppend: string
}

/**
 * Descriptor for a local path dependency.
 */
export type PackagePathDescriptor = PackageType & {
  /** The local path. */
  path: string
  /** The range of the path in the file. */
  pathRange: PackageTextRange
}

/**
 * Descriptor for a hosted dependency.
 */
export type PackageHostedDescriptor = PackageType & {
  /** The alias name of the package. */
  hostPackageName: string
  /** The URL of the host. */
  hostUrl: string
}

/**
 * Descriptor for a Git dependency.
 */
export type PackageGitDescriptor = PackageType & {
  /** The Git repository URL. */
  gitUrl: string
  /** Optional range of the Git specification. */
  gitRange?: PackageTextRange
  /** Optional Git reference (branch, tag, commit). */
  gitRef?: string
  /** Optional path within the Git repository. */
  gitPath?: string
}

/**
 * Descriptor for a GitHub dependency.
 */
export type PackageGitHubDescriptor = PackageType & {
  /** The GitHub repository URL. */
  githubUrl: string
  /** Optional range of the GitHub specification. */
  githubRange?: PackageTextRange
  /** Optional reference (branch, tag, commit). */
  githubRef?: string
}

/**
 * Descriptor for a package registry.
 */
export type PackageRegistryDescriptor = PackageType & {
  /** The registry URL. */
  registry: string
}

/**
 * Descriptor for a package group entry.
 */
export type PackageGroupDescriptor = PackageType & {
  /** The name of the group the entry is in. */
  groupName: string
  /** The full range of the entry associated with the group in the file. */
  range: PackageTextRange
}

/**
 * Descriptor indicating changes should be ignored.
 */
export type PackageIgnoreChangesDescriptor = PackageType & {}

/**
 * Descriptor for the project's own version.
 */
export type PackageProjectVersionDescriptor = PackageType & {}

/**
 * Descriptor for a Docker image.
 */
export type PackageImageDescriptor = PackageType & {
  /** The name component of the image. */
  nameDesc: PackageNameDescriptor
  /** The version (tag) component of the image. */
  versionDesc: PackageVersionDescriptor
  /** The registry component of the image. */
  registry?: string
}

/**
 * Descriptor for a Docker build configuration.
 */
export type PackageBuildDescriptor = PackageType & {
  /** The path to the Dockerfile or context. */
  pathDesc: PackagePathDescriptor
}

/**
 * Union type of all package descriptor types.
 */
export type PackageTypeDescriptor = PackageNameDescriptor
  | PackageVersionDescriptor
  | PackagePathDescriptor
  | PackageHostedDescriptor
  | PackageGitDescriptor
  | PackageGitHubDescriptor
  | PackageIgnoreChangesDescriptor
  | PackageProjectVersionDescriptor
  | PackageImageDescriptor
  | PackageBuildDescriptor
  | PackageRegistryDescriptor
  | PackageGroupDescriptor
