import {
  type PackageGitDescriptor,
  type PackageGitHubDescriptor,
  type PackageGroupDescriptor,
  type PackageHostedDescriptor,
  type PackageIgnoreChangesDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageProjectVersionDescriptor,
  type PackageRegistryDescriptor,
  type PackageTextRange,
  type PackageVersionDescriptor,
  PackageDescriptorType
} from '#domain/parsers';

/**
 * Creates a package name descriptor.
 * @param name The name of the package.
 * @param nameRange The range of the name in the text.
 * @returns A package name descriptor.
 */
export function createPackageNameDesc(name: string, nameRange: PackageTextRange): PackageNameDescriptor {
  return {
    type: PackageDescriptorType.name,
    name,
    nameRange
  };
}

/**
 * Creates a package version descriptor.
 * @param version The version string.
 * @param versionRange The range of the version in the text.
 * @param versionPrepend Text to prepend when updating.
 * @param versionAppend Text to append when updating.
 * @returns A package version descriptor.
 */
export function createPackageVersionDesc(
  version: string,
  versionRange: PackageTextRange,
  versionPrepend: string = '',
  versionAppend: string = ''
): PackageVersionDescriptor {
  return {
    type: PackageDescriptorType.version,
    version,
    versionRange,
    versionPrepend,
    versionAppend
  };
}

/**
 * Creates a Git dependency descriptor.
 * @param gitUrl The Git repository URL.
 * @param gitPath Optional path within the repository.
 * @param gitRef Optional reference (branch, tag, commit).
 * @param gitRange Optional range of the Git specification in the text.
 * @returns A Git descriptor.
 */
export function createPackageGitDescType(
  gitUrl: string,
  gitPath: string = '',
  gitRef: string = '',
  gitRange?: PackageTextRange
): PackageGitDescriptor {
  return {
    type: PackageDescriptorType.git,
    gitUrl,
    gitRef,
    gitPath,
    gitRange
  }
}

/**
 * Creates a GitHub dependency descriptor.
 * @param githubUrl The GitHub repository URL.
 * @param githubRef Optional reference (branch, tag, commit).
 * @param githubRange Optional range of the GitHub specification in the text.
 * @returns A GitHub descriptor.
 */
export function createPackageGitHubDescType(
  githubUrl: string,
  githubRef: string = '',
  githubRange?: PackageTextRange
): PackageGitHubDescriptor {
  return {
    type: PackageDescriptorType.github,
    githubUrl,
    githubRef,
    githubRange
  }
}

/**
 * Creates a local path dependency descriptor.
 * @param path The local path.
 * @param pathRange The range of the path in the text.
 * @returns A path descriptor.
 */
export function createPackagePathDescType(
  path: string,
  pathRange: PackageTextRange
): PackagePathDescriptor {
  return {
    type: PackageDescriptorType.path,
    path,
    pathRange
  }
}

/**
 * Creates a hosted dependency descriptor.
 * @param hostUrl The host URL.
 * @param hostPackageName The package name on the host.
 * @returns A hosted descriptor.
 */
export function createPackageHostedDescType(
  hostUrl: string,
  hostPackageName: string = '',
): PackageHostedDescriptor {
  return {
    type: PackageDescriptorType.hosted,
    hostPackageName,
    hostUrl
  }
}
/**
 * Creates an ignore changes descriptor.
 * @returns An ignore changes descriptor.
 */
export function createIgnoreChangesDesc(): PackageIgnoreChangesDescriptor {
  return { type: PackageDescriptorType.ignoreChanges }
}

/**
 * Creates a project version descriptor.
 * @returns A project version descriptor.
 */
export function createProjectVersionTypeDesc(): PackageProjectVersionDescriptor {
  return { type: PackageDescriptorType.projectVersion }
}

/**
 * Creates a package registry descriptor.
 * @param registry The registry URL.
 * @returns A registry descriptor.
 */
export function createPackageRegistryDescType(registry: string): PackageRegistryDescriptor {
  return {
    type: PackageDescriptorType.registry,
    registry
  };
}

/**
 * Creates a package group range descriptor.
 * @param groupName The name of the group.
 * @param range The full range of the entry.
 * @returns A group range descriptor.
 */
export function createPackageGroupDesc(
  groupName: string,
  range: PackageTextRange
): PackageGroupDescriptor {
  return {
    type: PackageDescriptorType.group,
    groupName,
    range
  };
}
