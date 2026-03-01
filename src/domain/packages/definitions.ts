import type { ClientResponseSource } from '#domain/clients';
import type { PackageDependency, PackageSuggestion } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';

/**
 * Enum representing the different types of package versions.
 */
export enum PackageVersionType {
  /** A fixed version (e.g., 1.2.3). */
  Version = 'version',
  /** A version range (e.g., ^1.2.3). */
  Range = 'range',
  /** A distribution tag (e.g., latest, beta). */
  Tag = 'tag',
  /** An alias to another package or version. */
  Alias = 'alias',
  /** A Git committish (branch, tag, or commit hash). */
  Committish = 'committish'
}

/**
 * Event handler type for package dependency changes.
 */
export type OnPackageDependenciesChangedEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
  packageDeps: PackageDependency[]
) => Promise<void>;

/**
 * Represents a basic package name and version pair.
 */
export type PackageNameVersion = {
  /** The name of the package. */
  name: string;
  /** The version of the package. */
  version: string;
};

/**
 * Represents a package name, version, and its physical path.
 */
export type PackageResource = PackageNameVersion & {
  /** The path to the package file or directory. */
  path: string;
};

/**
 * Represents a complete response for a package suggestion request.
 */
export type PackageResponse = {
  /** The name of the provider that generated the response. */
  providerName: string;
  /** The original parsed dependency. */
  parsedDependency: PackageDependency,
  /** The package data fetched from the registry. */
  fetchedPackage?: PackageNameVersion | null;
  /** The source of the package (e.g., Registry, Git). */
  packageSource: PackageSourceType;
  /** The type of the package version. */
  type: PackageVersionType | null;
  /** The specific suggestion for this package. */
  suggestion?: PackageSuggestion;
  /** The display order of the suggestion. */
  order: number;
};

/**
 * Represents lists of release and prerelease versions for a package.
 */
export type PackageVersions = {
  /** List of stable release versions. */
  releases: Array<string>
  /** List of prerelease versions. */
  prereleases: Array<string>
}

/**
 * Represents a parsed semver specification.
 */
export type SemverSpec = {
  /** The raw version string. */
  rawVersion: string
  /** The type of version (Version or Range). */
  type: PackageVersionType
};

/**
 * Enum representing the source of a package dependency.
 */
export enum PackageSourceType {
  /** Local directory dependency. */
  Directory = 'directory',
  /** Local file dependency. */
  File = 'file',
  /** Git repository dependency. */
  Git = 'git',
  /** GitHub repository dependency. */
  Github = 'github',
  /** Package registry dependency (e.g., npm, NuGet). */
  Registry = 'registry'
}

/**
 * Represents a request to fetch suggestions for a package.
 * @template TClientData Type of the provider-specific client data.
 */
export type PackageClientRequest<TClientData> = {
  /** The name of the suggestion provider. */
  providerName: string;
  /** Custom data specific to the provider. */
  clientData: TClientData,
  /** The parsed dependency to fetch suggestions for. */
  parsedDependency: PackageDependency;
};

/**
 * Represents the status of a package client response.
 */
export type PackageClientResponseStatus = {
  /** The source of the response (Remote, Cache, Local). */
  source: ClientResponseSource;
  /** The HTTP status code or custom status. */
  status: number;
  /** Whether the request was rejected. */
  rejected?: boolean;
};

/**
 * Represents the result of a package fetch operation.
 */
export type PackageClientResponse = {
  /** The source of the package. */
  source: PackageSourceType;
  /** The status of the response. */
  responseStatus?: PackageClientResponseStatus;
  /** The type of version requested. */
  type: PackageVersionType | null;
  /** The resolved package name and version if successful. */
  resolved?: PackageNameVersion | null;
  /** The list of suggestions generated for this package. */
  suggestions: Array<PackageSuggestion>;
  /** Optional Git-specific specification. */
  gitSpec?: any;
};