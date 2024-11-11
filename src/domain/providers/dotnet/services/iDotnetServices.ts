import { CachingOptions } from '#domain/caching';
import { IJsonHttpClient, IProcessClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
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

  dotnetProcess: IProcessClient;

  dotnetCli: DotNetCli;

  dotnetJsonClient: IJsonHttpClient;

  nugetClient: NuGetPackageClient;

  nugetResClient: NuGetResourceClient;

}