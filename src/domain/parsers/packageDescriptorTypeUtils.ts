import {
  type PackageGitDescriptor,
  type PackageHostedDescriptor,
  type PackageIgnoreChangesDescriptor,
  type PackageNameDescriptor,
  type PackageParentDescriptor,
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
 * @returns A Git descriptor.
 */
export function createPackageGitDescType(
  gitUrl: string,
  gitPath: string = '',
  gitRef: string = ''
): PackageGitDescriptor {
  return {
    type: PackageDescriptorType.git,
    gitUrl,
    gitPath,
    gitRef
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
 * Creates a parent path descriptor.
 * @param path The parent file path.
 * @returns A parent descriptor.
 */
export function createPackageParentDescType(path: string): PackageParentDescriptor {
  return {
    type: PackageDescriptorType.parent,
    path
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