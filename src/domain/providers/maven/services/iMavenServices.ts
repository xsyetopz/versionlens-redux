import { CachingOptions } from '#domain/caching';
import { HttpOptions, IHttpClient, IShellClient } from '#domain/clients';
import { MavenClient, MavenConfig, MvnCli } from '#domain/providers/maven';

export interface IMavenServices {

  mavenCachingOpts: CachingOptions;

  mavenHttpOpts: HttpOptions;

  mavenConfig: MavenConfig;

  mvnShellClient: IShellClient;

  mvnCli: MvnCli;

  mavenHttpClient: IHttpClient;

  mavenClient: MavenClient;

}