import {
  type PackageResponse,
  createPackageResource,
  PackageDependency,
  SuggestionCategory,
  SuggestionTypes
} from '#domain/packages';
import { PackageDescriptor } from '#domain/parsers';
import type { SuggestionsStats } from '#domain/useCases';

export default {
  test: <PackageResponse[]>[
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test1', '1.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Error
      }
    },
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test2', '2.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Error
      }
    },
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test3', '3.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Error
      }
    },
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test4', '4.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.NoMatch
      }
    },
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test5', '5.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Match
      }
    },
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test6', '6.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Latest
      }
    },
    {
      providerName: 'test-provider',
      parsedDependency: new PackageDependency(
        createPackageResource('test7', '7.0.0', 'test/path1'),
        new PackageDescriptor([])
      ),
      suggestion: {
        type: SuggestionTypes.status,
        category: SuggestionCategory.Directory
      }
    },
  ],
  expected: [
    <SuggestionsStats>{
      providerName: 'test-provider',
      filePath: 'test/path1',
      errors: 3,
      noMatches: 1,
      updates: 1,
    }
  ]
}