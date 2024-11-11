import { ClientResponseSource, HttpClientResponse } from '#domain/clients';

export function convertNpmErrorToResponse(
  error,
  source: ClientResponseSource
): HttpClientResponse {
  return {
    source,
    status: error.code,
    data: error.message,
  }
}