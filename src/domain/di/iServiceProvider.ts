import { IDisposable } from '#domain/utils';

export interface IServiceProvider extends IDisposable {

  name: string;

  getService: <T>(name: string) => T;

}