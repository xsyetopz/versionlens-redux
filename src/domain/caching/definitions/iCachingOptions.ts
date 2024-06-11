import { IFrozenOptions } from '#domain/configuration';

export interface ICachingOptions extends IFrozenOptions {

  config: IFrozenOptions;

  duration: number;

}
