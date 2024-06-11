import {
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageHostedDescriptor,
  TPackageIgnoreChangesDescriptor,
  TPackageNameDescriptor,
  TPackageParentDescriptor,
  TPackagePathDescriptor,
  TPackageProjectVersionDescriptor,
  TPackageTextRange,
  TPackageVersionDescriptor
} from '#domain/packages';

export function createPackageNameDesc(name: string, nameRange: TPackageTextRange): TPackageNameDescriptor {
  return {
    type: PackageDescriptorType.name,
    name,
    nameRange
  };
}

export function createPackageVersionDesc(
  version: string,
  versionRange: TPackageTextRange,
  versionPrepend: string = '',
  versionAppend: string = ''
): TPackageVersionDescriptor {
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
): TPackageGitDescriptor {
  return {
    type: PackageDescriptorType.git,
    gitUrl,
    gitPath,
    gitRef
  }
}

export function createPackagePathDescType(
  path: string,
  pathRange: TPackageTextRange
): TPackagePathDescriptor {
  return {
    type: PackageDescriptorType.path,
    path,
    pathRange
  }
}

export function createPackageHostedDescType(
  hostUrl: string,
  hostPackageName: string = '',
): TPackageHostedDescriptor {
  return {
    type: PackageDescriptorType.hosted,
    hostPackageName,
    hostUrl
  }
}

export function createPackageParentDescType(path: string): TPackageParentDescriptor {
  return {
    type: PackageDescriptorType.parent,
    path
  }
}

export function createIgnoreChangesDesc(): TPackageIgnoreChangesDescriptor {
  return { type: PackageDescriptorType.ignoreChanges }
}

export function createProjectVersionTypeDesc(): TPackageProjectVersionDescriptor {
  return { type: PackageDescriptorType.projectVersion }
}