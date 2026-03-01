import { ClientResponseSource } from './definitions';

/**
 * Error thrown when a shell client request fails.
 */
export class ShellClientRequestError extends Error {

  /**
   * Initializes a new instance of the ShellClientRequestError class.
   * @param message The error message.
   * @param cause The underlying error that caused the failure.
   */
  constructor(message: string, cause: Error) {
    super(`${ShellClientRequestError.name}:\n${message}`, { cause });
  }

  /**
   * The underlying error.
   */
  get cause(): Error { return <Error>super.cause };
}

/**
 * Error class representing an HTTP request failure.
 */
export class HttpRequestError {
  /**
   * Initializes a new instance of the HttpRequestError class.
   * @param source The source of the response.
   * @param status The HTTP status code.
   * @param data The response data or error message.
   * @param rejected Whether the request was rejected.
   */
  constructor(
    readonly source: ClientResponseSource,
    readonly status: number,
    readonly data: string,
    readonly rejected = true
  ) { }
}