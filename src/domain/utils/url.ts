import type { QueryDictionary } from '#domain/clients';
import { parse } from 'node:url';

export enum RegistryProtocols {
  file = 'file:',
  http = 'http:',
  https = 'https:',
}

type RegistryProtocolName = keyof typeof RegistryProtocols;

export function getProtocolFromUrl(url: string): RegistryProtocols {
  const sourceUrl = parse(url);
  const registryProtocol = sourceUrl.protocol === null
    ? RegistryProtocols.file
    : RegistryProtocols[sourceUrl.protocol.substring(0, sourceUrl.protocol.length - 1) as RegistryProtocolName];

  return registryProtocol || RegistryProtocols.file;
}

export function createUrl(baseUrl: string, queryParams: QueryDictionary): string {
  const query = buildQueryParams(queryParams);

  const slashedUrl = query.length > 0
    ? trimEndSlash(baseUrl)
    : baseUrl;

  return slashedUrl + query;
}

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

export function trimEndSlash(url: string): string {
  let result = url;
  while (result.endsWith('/')) {
    result = result.slice(0, -1)
  }
  return result;
}

export function ensureEndSlash(url: string): string {
  return url.endsWith('/') ? url : url + '/';
}