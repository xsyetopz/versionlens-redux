import { PackageDependency, PackageSourceType, TPackageSuggestion } from '#domain/packages';
import { ISuggestionProvider } from '#domain/providers';
import { Uri } from "vscode";

export enum PackageVersionType {
  Version = 'version',
  Range = 'range',
  Tag = 'tag',
  Alias = 'alias',
  Committish = 'committish'
}

export type OnPackageDependenciesChangedEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
  packageDeps: PackageDependency[]
) => Promise<void>;

export interface IPackageFileWatcher {
  watchFolder(): Promise<void>;
  watchFile(file: Uri): Promise<void>
  watch: () => void;
  registerListener: (
    listener: OnPackageDependenciesChangedEvent,
    thisArg: any
  ) => void;
}

export type TPackageNameVersion = {
  name: string;
  version: string;
};

export type TPackageResource = TPackageNameVersion & {
  path: string;
};

export type PackageResponse = {
  providerName: string;
  parsedDependency: PackageDependency,
  fetchedPackage?: TPackageNameVersion;
  packageSource?: PackageSourceType;
  type?: PackageVersionType;
  suggestion?: TPackageSuggestion;
  order: number;
};

export type TPackageVersions = {
  releases: Array<string>,
  prereleases: Array<string>
}

export type TSemverSpec = {
  rawVersion: string,
  type: PackageVersionType,
};