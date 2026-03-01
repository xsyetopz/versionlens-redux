import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { PnpmConfig } from '#domain/providers/pnpm';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for PNPM configuration.
 */
export enum PnpmFeatures {
  Caching = 'pnpm.caching',
  Http = 'pnpm.http',
  DependencyProperties = 'pnpm.dependencyProperties',
  FilePatterns = 'pnpm.files'
}

/**
 * Defines the services provided by the PNPM provider.
 */
export interface IPnpmServices {
  /**
   * Caching options for PNPM.
   */
  pnpmCachingOpts: CachingOptions
  /**
   * HTTP options for PNPM.
   */
  pnpmHttpOpts: HttpOptions
  /**
   * Configuration for PNPM.
   */
  pnpmConfig: PnpmConfig
}

/**
 * Service name constant for PNPM services.
 */
export const PnpmService = nameOf<IPnpmServices>()