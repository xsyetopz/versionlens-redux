import type { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { PackageDependency, PackageSuggestion } from '#domain/packages';
import type { IProviderConfig, ISuggestionProvider } from '#domain/providers';

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
  suggestion?: PackageSuggestion;
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

export enum PackageSourceType {
  Directory = 'directory',
  File = 'file',
  Git = 'git',
  Github = 'github',
  Registry = 'registry'
}

export interface IPackageClient<TClientData> {
  logger: ILogger;
  config: IProviderConfig,
  fetchPackage: (request: TPackageClientRequest<TClientData>)
    => Promise<TPackageClientResponse>;
}

export type TPackageClientRequest<TClientData> = {
  // provider descriptor
  providerName: string;
  // provider specific data
  clientData: TClientData,
  // dependency to fetch
  parsedDependency: PackageDependency;
  // number of fallback attempts
  attempt: number;
};

export type TPackageClientResponseStatus = {
  source: ClientResponseSource;
  status: number;
  rejected?: boolean;
};

export type TPackageClientResponse = {
  source: PackageSourceType;
  responseStatus?: TPackageClientResponseStatus;
  type: PackageVersionType;
  resolved?: TPackageNameVersion;
  suggestions: Array<PackageSuggestion>;
  gitSpec?: any;
};