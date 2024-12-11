import {
  type PackageSuggestion,
  PackageStatusFactory,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes
} from '#domain/packages';
import assert from 'node:assert';

export const PackageStatusFactoryTests = {

  [PackageStatusFactory.createFromHttpStatus.name]: {

    "returns suggestions from implemented http status $1": [
      [
        400,
        <PackageSuggestion>{
          name: SuggestionStatusText.BadRequest,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }
      ],
      [
        401,
        <PackageSuggestion>{
          name: SuggestionStatusText.NotAuthorized,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }
      ],
      [
        403,
        <PackageSuggestion>{
          name: SuggestionStatusText.Forbidden,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }
      ],
      [
        404,
        <PackageSuggestion>{
          name: SuggestionStatusText.NotFound,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }
      ],
      [
        500,
        <PackageSuggestion>{
          name: SuggestionStatusText.InternalServerError,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }
      ],
      (testStatus: number, expected: PackageSuggestion) => {
        const actual = PackageStatusFactory.createFromHttpStatus(testStatus)
        assert.deepEqual(actual, expected)
      }
    ],

    "returns null when http status not implemented": () => {
      const actual = PackageStatusFactory.createFromHttpStatus(501)
      assert.deepEqual(actual, null)
    }

  }

}