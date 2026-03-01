import { XHROptions } from 'request-light';

/**
 * Represents a response from the low-level XHR request.
 */
export interface IXhrResponse {
  /** The response body as a string. */
  responseText: string;
  /** The HTTP status code. */
  status: number;
  /** The response headers. */
  headers: any;
}

/**
 * Interface for the low-level XHR request function.
 */
export interface IXhrRequest {
  /** Performs an XHR request. */
  xhr: (options: XHROptions) => Promise<IXhrResponse>;
}