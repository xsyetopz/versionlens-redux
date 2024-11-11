import { TPromiseSpawnOptions, TPromiseSpawnResult } from '#domain/process/promiseSpawn';

export class ProcessSpawnStub {

  promiseSpawn(
    cmd: string,
    args?: Array<string>,
    opts?: TPromiseSpawnOptions,
    // extra?: any
  ): Promise<TPromiseSpawnResult> {
    return Promise.resolve() as any;
  }

}