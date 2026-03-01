import type { QueryDictionary } from '#domain/clients';
import { parse } from 'node:url';

/**
 * Enum representing supported registry protocols.
 */
export enum RegistryProtocols {
  /** Local file system. */
  file = 'file:',
  /** HTTP protocol. */
  http = 'http:',
  /** HTTPS protocol. */
  https = 'https:',
}

type RegistryProtocolName = keyof typeof RegistryProtocols;

/**
 * Extracts the protocol from a URL string.
 * @param url The URL string.
 * @returns The matching RegistryProtocols enum value, or file: if no match found.
 */
export function getProtocolFromUrl(url: string): RegistryProtocols {
  const sourceUrl = parse(url);
  const registryProtocol = sourceUrl.protocol === null
    ? RegistryProtocols.file
    : RegistryProtocols[sourceUrl.protocol.substring(0, sourceUrl.protocol.length - 1) as RegistryProtocolName];

  return registryProtocol || RegistryProtocols.file;
}

/**
 * Creates a URL string from a base URL and query parameters.
 * @param baseUrl The base URL.
 * @param queryParams A dictionary of query parameters.
 * @returns The combined URL string.
 */
export function createUrl(baseUrl: string, queryParams: QueryDictionary): string {
  const query = buildQueryParams(queryParams);

  const slashedUrl = query.length > 0
    ? trimEndSlash(baseUrl)
    : baseUrl;

  return slashedUrl + query;
}

/**
 * Internal function to build a query string from a dictionary.
 */
function buildQueryParams(queryParams: QueryDictionary): string {
  let query = '';
  if (queryParams) {
    query = Object.keys(queryParams)
      .map(key => `${key}=${queryParams[key]}`)
      .join('&');
    query = (query.length > 0) ? '?' + query : '';
  }
  return query;
}

/**
 * Trims all trailing slashes from a URL string.
 * @param url The URL string.
 * @returns The URL without trailing slashes.
 */
export function trimEndSlash(url: string): string {
  let result = url;
  while (result.endsWith('/')) {
    result = result.slice(0, -1)
  }
  return result;
}

/**
 * Ensures a URL string has exactly one trailing slash.
 * @param url The URL string.
 * @returns The URL with a trailing slash.
 */
export function ensureEndSlash(url: string): string {
  return url.endsWith('/') ? url : url + '/';
}