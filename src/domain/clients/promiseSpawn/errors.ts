export class ShellClientRequestError extends Error {

  constructor(message: string, cause: Error) {
    super(`${ShellClientRequestError.name}:\n${message}`, { cause });
  }

  get cause(): Error { return <Error>super.cause };
}