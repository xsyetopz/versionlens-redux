import { throwUndefinedOrNull } from '@esm-test/guards';
import { IFrozenOptions, IOptions } from '#domain/configuration';
import { Undefinable } from 'domain/utils';

export abstract class Options implements IOptions {

  constructor(
    readonly config: IFrozenOptions, 
    protected section: string
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("section", section);

    this.config = config;
    this.section = (section.length > 0) ? section + '.' : '';
  }

  get<T>(key: string): Undefinable<T> {
    return this.config.get(`${this.section}${key}`);
  }

  defrost(): void {
    this.config.defrost();
  }

}