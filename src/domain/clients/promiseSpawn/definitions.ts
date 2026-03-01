/**
 * Options for the promise-spawn function.
 */
type PromiseSpawnOptions = {
  /** The current working directory. */
  cwd?: string,
  /** Whether to capture stdout/stderr as strings. */
  stdioString?: boolean,
  /** Stdio configuration. */
  stdio?: string
};

/**
 * Represents the result of a spawned process.
 */
type PromiseSpawnResult = {
  /** The exit code of the process. */
  code: any,
  /** Captured stdout data. */
  stdout: any,
  /** Captured stderr data. */
  stderr: any,
  /** The signal that terminated the process. */
  signal: any,
  /** Additional data. */
  extra: any
};

/**
 * Function signature for spawning a process and returning a promise.
 */
export type PromiseSpawnFn = (
  cmd: string,
  args?: Array<string>,
  opts?: PromiseSpawnOptions,
  extra?: any
) => Promise<PromiseSpawnResult>;