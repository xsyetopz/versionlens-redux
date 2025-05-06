import type { IFrozenOptions, IOptions } from '#domain/configuration';
import { throwUndefinedOrNull } from '@esm-test/guards';

export abstract class Options implements IOptions {

  constructor(
    readonly config: IFrozenOptions,
    protected section: string
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("section", section);

    this.section = (section.length > 0) ? section + '.' : '';
  }

  get<T>(key: string): T | undefined {
    return this.config.get(`${this.section}${key}`);
  }

  getOrDefault<T>(key: string, defaultValue: T): T {
    // attempt to get the section value
    const value = this.get<T>(key);

    // return key value
    if (value !== null && value !== undefined) return value;

    // return default value
    return defaultValue;
  }

  defrost(): void {
    this.config.defrost();
  }

}