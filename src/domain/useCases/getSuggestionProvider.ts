import { throwUndefinedOrNull } from '@esm-test/guards';
import { ISuggestionProvider } from '#domain/providers';
import { isMatch } from 'micromatch';
import { basename } from 'node:path';

export class GetSuggestionProvider {

  constructor(private readonly suggestionProviders: Array<ISuggestionProvider>) {
    throwUndefinedOrNull("suggestionProviders", suggestionProviders);
  }

  execute(filePath: string): ISuggestionProvider {
    const filename = basename(filePath);

    let filtered = this.suggestionProviders
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