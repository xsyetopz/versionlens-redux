import { HttpFeatures, IHttpOptions } from '#domain/clients';
import { IFrozenOptions, OptionsWithFallback } from '#domain/configuration';
import { Nullable } from '#domain/utils';

export class HttpOptions extends OptionsWithFallback implements IHttpOptions {

  constructor(
    config: IFrozenOptions,
    section: string,
    fallbackSection: Nullable<string> = null
  ) {
    super(config, section, fallbackSection);
  }

  get strictSSL(): boolean {
    return this.getOrDefault<boolean>(
      HttpFeatures.StrictSSL,
      true
    );
  }

}