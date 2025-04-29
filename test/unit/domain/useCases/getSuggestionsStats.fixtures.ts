import { type PackageResponse, SuggestionCategory, SuggestionTypes } from '#domain/packages';
import type { SuggestionsStats } from '#domain/useCases';

export default {
  test: <PackageResponse[]>[
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Error
      }
    },
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Error
      }
    },
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Error
      }
    },
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.NoMatch
      }
    },
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Match
      }
    },
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Latest
      }
    },
    {
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Directory
      }
    },


  ],
  expected: [
    <SuggestionsStats>{
      providerName: 'test1',
      filePath: 'test/path1',
      errors: 3,
      noMatches: 1,
      updates: 1,
    }
  ]
}