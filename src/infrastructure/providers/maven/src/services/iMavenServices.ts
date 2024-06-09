import { CachingOptions } from "domain/caching";
import { IHttpClient, IProcessClient } from "domain/clients";
import { HttpOptions } from "domain/http";
import { MavenClient, MvnCli } from "../index";
import { MavenConfig } from "../mavenConfig";

export interface IMavenServices {

  mavenCachingOpts: CachingOptions;

  mavenHttpOpts: HttpOptions;

  mavenConfig: MavenConfig;

  mvnProcess: IProcessClient;

  mvnCli: MvnCli;

  mavenHttpClient: IHttpClient;

  mavenClient: MavenClient;

}