import { CachingOptions } from "domain/caching";
import { IHttpClient } from "domain/clients";
import { HttpOptions } from "domain/http";
import { GoClient } from "../goClient";
import { GoConfig } from "../goConfig";

export interface IGoService {

  goCachingOpts: CachingOptions;

  goHttpOpts: HttpOptions;

  goConfig: GoConfig;

  goHttpClient: IHttpClient;

  goClient: GoClient;

}