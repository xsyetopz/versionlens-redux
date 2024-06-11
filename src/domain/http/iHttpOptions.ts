import { IFrozenOptions } from '#domain/configuration';

export interface IHttpOptions extends IFrozenOptions {

  config: IFrozenOptions;

  strictSSL: boolean;

}