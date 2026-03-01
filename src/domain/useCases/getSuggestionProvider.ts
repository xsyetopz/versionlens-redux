import type { ISuggestionProvider } from '#domain/providers';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { isMatch } from 'micromatch';
import { basename } from 'node:path';

/**
 * Use case for identifying the correct suggestion provider for a given file.
 */
export class GetSuggestionProvider {

  /**
   * Initializes a new instance of the GetSuggestionProvider class.
   * @param suggestionProviders The list of available suggestion providers.
   */
  constructor(private readonly suggestionProviders: Array<ISuggestionProvider>) {
    throwUndefinedOrNull("suggestionProviders", suggestionProviders);
  }

  /**
   * Identifies the suggestion provider for a file based on its path.
   * @param filePath The path to the package file.
   * @returns The matching suggestion provider, or undefined if no match is found.
   */
  execute(filePath: string): ISuggestionProvider | undefined {
    const filename = basename(filePath);

    const filtered = this.suggestionProviders
      .filter(
        provider => isMatch(filename, provider.config.filePatterns)
      )
      .filter(
        provider => !(
          provider.config.fileExcludePatterns &&
          isMatch(filePath, provider.config.fileExcludePatterns)
        )
      );

    if (filtered.length === 0) return;

    return filtered[0];
  }

}