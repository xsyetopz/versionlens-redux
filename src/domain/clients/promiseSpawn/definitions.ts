export type TPromiseSpawnOptions = {
  cwd?: string,
  stdioString?: boolean,
  stdio?: string
};

export type TPromiseSpawnResult = {
  code: any,
  stdout: any,
  stderr: any,
  signal: any,
  extra: any
};

export type PromiseSpawnFn = (
  cmd: string,
  args?: Array<string>,
  opts?: TPromiseSpawnOptions,
  extra?: any
) => Promise<TPromiseSpawnResult>;