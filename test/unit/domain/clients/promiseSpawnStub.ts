import { TPromiseSpawnOptions, TPromiseSpawnResult } from '#domain/clients/promiseSpawn';

export class PromiseSpawnStub {

  promiseSpawn(
    cmd: string,
    args?: Array<string>,
    opts?: TPromiseSpawnOptions,
    // extra?: any
  ): Promise<TPromiseSpawnResult> {
    return Promise.resolve() as any;
  }

}