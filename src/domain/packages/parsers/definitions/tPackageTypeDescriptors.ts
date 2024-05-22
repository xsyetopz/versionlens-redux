import { TPackageTextRange } from "domain/packages";

export type TPackageType = {
  type: string
}

export type TPackageNameDescriptor = TPackageType & {
  name: string
  nameRange: TPackageTextRange
}

export type TPackageVersionDescriptor = TPackageType & {
  version: string
  versionRange: TPackageTextRange
  versionPrepend: string
  versionAppend: string
}

export type TPackagePathDescriptor = TPackageType & {
  path: string
  pathRange: TPackageTextRange
}

export type TPackageHostedDescriptor = TPackageType & {
  hostPackageName: string
  hostUrl: string
}

export type TPackageGitDescriptor = TPackageType & {
  gitUrl: string
  gitRef: string
  gitPath: string
}

export type TPackageParentDescriptor = TPackageType & {
  path: string
}

export type TPackageTypeDescriptor = TPackageNameDescriptor
  | TPackageVersionDescriptor
  | TPackagePathDescriptor
  | TPackageHostedDescriptor
  | TPackageGitDescriptor
  | TPackageParentDescriptor