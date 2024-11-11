import { CachingOptions } from '#domain/caching';
import { HttpOptions, IHttpClient, IProcessClient } from '#domain/clients';
import { MavenClient, MavenConfig, MvnCli } from '#domain/providers/maven';

export interface IMavenServices {

  mavenCachingOpts: CachingOptions;

  mavenHttpOpts: HttpOptions;

  mavenConfig: MavenConfig;

  mvnProcess: IProcessClient;

  mvnCli: MvnCli;

  mavenHttpClient: IHttpClient;

  mavenClient: MavenClient;

}