import type { CachingOptions } from '#domain/caching';
import type {
  HttpOptions,
  IJsonHttpClient,
  IShellClient,
  JsonClientResponse
} from '#domain/clients';
import type { PackageVersionType } from '#domain/packages';
import type {
  DotNetCli,
  DotNetConfig,
  DotnetClient,
  NuGetClient,
  NugetOptions
} from '#domain/providers/dotnet';
import { type RegistryProtocols, nameOf } from '#domain/utils';

export enum DotNetFeatures {
  Caching = 'dotnet.caching',
  Http = 'dotnet.http',
  Nuget = 'dotnet.nuget',
  DependencyProperties = 'dotnet.dependencyProperties',
  FilePatterns = 'dotnet.files',
  OnSaveChangesTask = 'dotnet.onSaveChanges',
  PrereleaseTagFilter = 'dotnet.prereleaseTagFilter',
}

export enum NugetFeatures {
  Sources = 'sources',
}

export type NugetVersionSpec = {
  version?: string;
  hasFourSegments?: boolean;
  isMinInclusive?: boolean;
  isMaxInclusive?: boolean;
  minVersionSpec?: NugetVersionSpec;
  maxVersionSpec?: NugetVersionSpec;
};

export type DotNetVersionSpec = {
  type: PackageVersionType,
  rawVersion: string,
  resolvedVersion: string,
  spec: NugetVersionSpec,
};

export type DotNetSource = {
  enabled: boolean,
  machineWide: boolean,
  url: string,
  protocol: RegistryProtocols,
}

export interface NugetServiceResource {
  '@id': string;
  '@type': string;
}

export interface NugetServiceIndex {
  version: string;
  resources: Array<NugetServiceResource>;
}

export type NugetServiceIndexResponse = JsonClientResponse<NugetServiceIndex>

export type NuGetClientData = {
  serviceUrls: Array<string>,
}

export interface IDotNetServices {
  dotnetCachingOpts: CachingOptions;
  dotnetHttpOpts: HttpOptions;
  nugetOpts: NugetOptions;
  dotnetConfig: DotNetConfig;
  dotnetShellClient: IShellClient;
  dotnetCli: DotNetCli;
  dotnetJsonClient: IJsonHttpClient;
  nugetClient: NuGetClient;
  dotnetClient: DotnetClient;
}

export const DotNetService = nameOf<IDotNetServices>()

export type NugetApiResult = {
  versions: string[]
}

export type NugetApiResponse = JsonClientResponse<NugetApiResult>

export type JsrClientResponse = JsonClientResponse<string[]>