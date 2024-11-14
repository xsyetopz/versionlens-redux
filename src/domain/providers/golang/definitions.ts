import { CachingOptions } from '#domain/caching';
import { HttpOptions, IHttpClient } from '#domain/clients';
import { GoClient, GoConfig } from '#domain/providers/golang';

export enum GoFeatures {
  Caching = 'golang.caching',
  Http = 'golang.http',
  ApiUrl = 'golang.apiUrl',
  FilePatterns = 'golang.files',
  OnSaveChangesTask = 'golang.onSaveChanges',
  PrereleaseTagFilter = 'golang.prereleaseTagFilter',
}

export interface IGoService {
  goCachingOpts: CachingOptions;
  goHttpOpts: HttpOptions;
  goConfig: GoConfig;
  goHttpClient: IHttpClient;
  goClient: GoClient;
}