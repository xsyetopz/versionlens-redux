import { IOptions } from '#domain/configuration';

export interface INugetOptions extends IOptions {

  sources: Array<string>;

}