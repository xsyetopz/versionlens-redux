import { CachingOptions } from '#domain/caching';
import { HttpOptions, IJsonHttpClient, IShellClient } from '#domain/clients';
import {
  DotNetCli,
  DotNetConfig,
  NuGetPackageClient,
  NuGetResourceClient,
  NugetOptions
} from '#domain/providers/dotnet';

export interface IDotNetServices {

  dotnetCachingOpts: CachingOptions;

  dotnetHttpOpts: HttpOptions;

  nugetOpts: NugetOptions;

  dotnetConfig: DotNetConfig;

  dotnetShellClient: IShellClient;

  dotnetCli: DotNetCli;

  dotnetJsonClient: IJsonHttpClient;

  nugetClient: NuGetPackageClient;

  nugetResClient: NuGetResourceClient;

}