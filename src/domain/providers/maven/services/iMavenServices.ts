import { CachingOptions } from '#domain/caching';
import { IHttpClient, IProcessClient } from '#domain/clients';
import { HttpOptions } from '#domain/http';
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