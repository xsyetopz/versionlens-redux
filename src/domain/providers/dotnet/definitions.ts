import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { PackageVersionType } from '#domain/packages';
import type {
  DotNetCli,
  DotNetConfig,
  DotnetSuggestionResolver,
  NuGetClient,
  NugetOptions
} from '#domain/providers/dotnet';
import { type RegistryProtocols, nameOf } from '#domain/utils';

/**
 * Feature keys used for DotNet configuration.
 */
export enum DotNetFeatures {
  Caching = 'dotnet.caching',
  Http = 'dotnet.http',
  Nuget = 'dotnet.nuget',
  DependencyProperties = 'dotnet.dependencyProperties',
  FilePatterns = 'dotnet.files',
  OnSaveChangesTask = 'dotnet.onSaveChanges',
  PrereleaseTagFilter = 'dotnet.prereleaseTagFilter',
}

/**
 * Feature keys used for NuGet configuration.
 */
export enum NugetFeatures {
  Sources = 'sources',
}

/**
 * Represents a NuGet version specification.
 */
export type NugetVersionSpec = {
  /**
   * The version string.
   */
  version?: string;
  /**
   * Whether the version has four segments (e.g., 1.2.3.4).
   */
  hasFourSegments?: boolean;
  /**
   * Whether the minimum version is inclusive.
   */
  isMinInclusive?: boolean;
  /**
   * Whether the maximum version is inclusive.
   */
  isMaxInclusive?: boolean;
  /**
   * The minimum version specification.
   */
  minVersionSpec?: NugetVersionSpec;
  /**
   * The maximum version specification.
   */
  maxVersionSpec?: NugetVersionSpec;
};

/**
 * Represents a resolved DotNet version specification.
 */
export type DotNetVersionSpec = {
  /**
   * The type of package version (Version, Range, or null).
   */
  type: PackageVersionType | null,
  /**
   * The raw version string from the file.
   */
  rawVersion: string,
  /**
   * The resolved version string (e.g., converted to semver).
   */
  resolvedVersion: string,
  /**
   * The parsed NuGet version specification.
   */
  spec: NugetVersionSpec | null,
};

/**
 * Represents a DotNet package source.
 */
export type DotNetSource = {
  /**
   * Whether the source is enabled.
   */
  enabled: boolean,
  /**
   * Whether the source is configured at the machine-wide level.
   */
  machineWide: boolean,
  /**
   * The URL of the source.
   */
  url: string,
  /**
   * The registry protocol (e.g., https, file).
   */
  protocol: RegistryProtocols,
}

/**
 * Represents a resource in the NuGet service index.
 */
export interface NugetServiceResource {
  '@id': string;
  '@type': string;
}

/**
 * Represents the NuGet service index response.
 */
export interface NugetServiceIndex {
  version: string;
  resources: Array<NugetServiceResource>;
}

/**
 * Represents the JSON response for the NuGet service index.
 */
export type NugetServiceIndexResponse = JsonClientResponse<NugetServiceIndex>

/**
 * Custom data passed to the NuGet client.
 */
export type NuGetClientData = {
  /**
   * The list of resolved service URLs for package base address.
   */
  serviceUrls: Array<string>,
}

/**
 * Defines the services provided by the DotNet provider.
 */
export interface IDotNetServices {
  /**
   * Caching options for DotNet.
   */
  dotnetCachingOpts: CachingOptions;
  /**
   * HTTP options for DotNet.
   */
  dotnetHttpOpts: HttpOptions;
  /**
   * NuGet-specific options.
   */
  nugetOpts: NugetOptions;
  /**
   * Configuration for DotNet.
   */
  dotnetConfig: DotNetConfig;
  /**
   * Client for the DotNet CLI.
   */
  dotnetCli: DotNetCli;
  /**
   * Client for the NuGet registry.
   */
  nugetClient: NuGetClient;
  /**
   * Resolver for DotNet suggestions.
   */
  dotnetSuggestionResolver: DotnetSuggestionResolver;
}

/**
 * Service name constant for DotNet services.
 */
export const DotNetService = nameOf<IDotNetServices>()

/**
 * Represents the package versions returned by the NuGet API.
 */
export type NugetApiResult = {
  /**
   * The array of version strings.
   */
  versions: string[]
}

/**
 * Represents the JSON response from the NuGet package API.
 */
export type NugetApiResponse = JsonClientResponse<NugetApiResult>