export enum PackageDescriptorType {
  name = "name",
  version = "version",
  path = "path",
  git = "git",
  hosted = "hosted",
  parent = "parent",
  ignoreChanges = "ignoreChanges",
  projectVersion = "projectVersion",
  image = "image",
  build = 'build',
  registry = 'registry'
}

export type PackageTextRange = {
  start: number;
  end: number;
};

export type PackageType = {
  type: keyof typeof PackageDescriptorType
}

export type PackageNameDescriptor = PackageType & {
  name: string
  nameRange: PackageTextRange
}

export type PackageVersionDescriptor = PackageType & {
  version: string
  versionRange: PackageTextRange
  versionPrepend: string
  versionAppend: string
}

export type PackagePathDescriptor = PackageType & {
  path: string
  pathRange: PackageTextRange
}

export type PackageHostedDescriptor = PackageType & {
  hostPackageName: string
  hostUrl: string
}

export type PackageGitDescriptor = PackageType & {
  gitUrl: string
  gitRef?: string
  gitPath?: string
}

export type PackageRegistryDescriptor = PackageType & {
  registry: string
}

export type PackageParentDescriptor = PackageType & {
  path: string
}

export type PackageIgnoreChangesDescriptor = PackageType & {}

export type PackageProjectVersionDescriptor = PackageType & {}

export type PackageImageDescriptor = PackageType & {
  nameDesc: PackageNameDescriptor
  versionDesc: PackageVersionDescriptor
  registry?: string
}

export type PackageBuildDescriptor = PackageType & {
  pathDesc: PackagePathDescriptor
}

export type PackageTypeDescriptor = PackageNameDescriptor
  | PackageVersionDescriptor
  | PackagePathDescriptor
  | PackageHostedDescriptor
  | PackageGitDescriptor
  | PackageParentDescriptor
  | PackageIgnoreChangesDescriptor
  | PackageProjectVersionDescriptor
  | PackageImageDescriptor
  | PackageBuildDescriptor
  | PackageRegistryDescriptor