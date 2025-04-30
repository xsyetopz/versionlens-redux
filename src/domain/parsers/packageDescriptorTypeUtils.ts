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

export function createPackageNameDesc(name: string, nameRange: PackageTextRange): PackageNameDescriptor {
  return {
    type: PackageDescriptorType.name,
    name,
    nameRange
  };
}

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

export function createPackageParentDescType(path: string): PackageParentDescriptor {
  return {
    type: PackageDescriptorType.parent,
    path
  }
}

export function createIgnoreChangesDesc(): PackageIgnoreChangesDescriptor {
  return { type: PackageDescriptorType.ignoreChanges }
}

export function createProjectVersionTypeDesc(): PackageProjectVersionDescriptor {
  return { type: PackageDescriptorType.projectVersion }
}

export function createPackageRegistryDescType(registry: string): PackageRegistryDescriptor {
  return {
    type: PackageDescriptorType.registry,
    registry
  };
}