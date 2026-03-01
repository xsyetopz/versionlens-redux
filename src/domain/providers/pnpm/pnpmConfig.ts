import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { PnpmFeatures } from '#domain/providers/pnpm';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the PNPM package provider.
 */
export class PnpmConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the PnpmConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for PNPM.
   * @param http The HTTP options for PNPM.
   */
  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('caching', caching);
    throwUndefinedOrNull('http', http);
  }

  /**
   * The file languages supported by this provider.
   */
  readonly fileLanguage = ['yaml'];

  /**
   * The task to run when a PNPM file is saved.
   */
  onSaveChangesTask = null;

  /**
   * Gets the file patterns used to identify PNPM files.
   */
  get filePatterns(): string {
    return this.config.get(PnpmFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in PNPM files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(PnpmFeatures.DependencyProperties, []);
  }

}