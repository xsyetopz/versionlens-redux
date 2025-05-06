type PromiseSpawnOptions = {
  cwd?: string,
  stdioString?: boolean,
  stdio?: string
};

type PromiseSpawnResult = {
  code: any,
  stdout: any,
  stderr: any,
  signal: any,
  extra: any
};

export type PromiseSpawnFn = (
  cmd: string,
  args?: Array<string>,
  opts?: PromiseSpawnOptions,
  extra?: any
) => Promise<PromiseSpawnResult>;