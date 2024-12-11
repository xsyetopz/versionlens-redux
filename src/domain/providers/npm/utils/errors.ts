import { ClientResponseSource, HttpClientResponse } from '#domain/clients';
import {
  type PackageSuggestion,
  PackageStatusFactory,
  SuggestionCategory,
  SuggestionTypes,
  UpdateableFactory
} from '#domain/packages';

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

export function createNpmSuggestionFromErrorCode(npmErrorCode: string): PackageSuggestion[] {
  switch (npmErrorCode) {
    case 'ECONNREFUSED':
      return [PackageStatusFactory.createConnectionRefusedStatus()];
    case 'ECONNRESET':
      return [PackageStatusFactory.createConnectionResetStatus()];
    case 'EUNSUPPORTEDPROTOCOL':
      return [PackageStatusFactory.createNotSupportedStatus()];
    case 'EINVALIDTAGNAME':
      return [
        PackageStatusFactory.createInvalidStatus(''),
        UpdateableFactory.createLatestUpdateable('latest')
      ];
    case 'EINVALIDPACKAGENAME':
      return [PackageStatusFactory.createInvalidStatus('')];
    default:
      const errorNum = Number.parseInt(npmErrorCode.substring(1));
      if (Number.isNaN(errorNum)) {
        return [{
          name: npmErrorCode,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }];
      }

      return [PackageStatusFactory.createFromHttpStatus(errorNum)];
  }
}