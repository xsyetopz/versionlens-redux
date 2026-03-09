import {
  type SuggestionUpdate,
  PackageSourceType
} from '#domain/packages';

const githubRefRegex = /(ref|tag|branch):\s*(['"])(?<ref>[^'"]*)\2/i;

/**
 * Custom function to replace versions in Ruby Gemfiles, handling GitHub and version operators.
 * @param suggestionUpdate The suggestion update information.
 * @param operatorRegex The regex to match version operators.
 * @returns The new version string.
 */
export function rubyReplaceVersion(
  suggestionUpdate: SuggestionUpdate,
  operatorRegex: RegExp
): string {

  if (suggestionUpdate.packageSource === PackageSourceType.Github) {
    return replaceRubyGitVersion(suggestionUpdate);
  }

  const suggestionVersion = suggestionUpdate.suggestionVersion;

  const insert = suggestionUpdate.parsedVersionPrepend.length > 1;
  if (insert) {
    return `${suggestionUpdate.parsedVersionPrepend}${suggestionVersion}${suggestionUpdate.parsedVersionAppend}`;
  }

  const match = operatorRegex.exec(suggestionUpdate.parsedVersion);
  if (match) {
    const operator = match[0];
    return `${operator}${suggestionVersion}`;
  }

  return suggestionVersion;
}

/**
 * Replaces a version in a Ruby Git dependency string.
 * @param suggestionUpdate The suggestion update information.
 * @returns The updated dependency string.
 */
function replaceRubyGitVersion(suggestionUpdate: SuggestionUpdate): string {
  if (!suggestionUpdate.fetchedVersion && suggestionUpdate.fetchedVersion !== '') return suggestionUpdate.parsedVersion;

  const suggestionVersion = suggestionUpdate.suggestionVersion;

  // check for ref, tag, or branch options
  const match = githubRefRegex.exec(suggestionUpdate.parsedVersion);
  if (match) {
    const option = match[1];
    const quote = match[2];
    const oldRef = match.groups!.ref;
    
    // if we are updating to a commit SHA, we should use 'ref:'
    // We use length as a simple heuristic for SHAs since it won't have dots
    const isSha = !suggestionVersion.includes('.') && suggestionVersion.length >= 7 && /^[0-9a-f]+$/i.test(suggestionVersion);

    const newOption = isSha ? 'ref' : option;

    const find = `${option}: ${quote}${oldRef}${quote}`;
    const replace = `${newOption}: ${quote}${suggestionVersion}${quote}`;

    // If it didn't find the exact string (e.g. whitespace differences), 
    // we use the full match to replace
    if (suggestionUpdate.parsedVersion.indexOf(find) === -1) {
      return suggestionUpdate.parsedVersion.replace(
        match[0],
        `${newOption}: ${quote}${suggestionVersion}${quote}`
      );
    }

    return suggestionUpdate.parsedVersion.replace(find, replace);
  }

  // if no ref/tag/branch option found, it might be just github: 'user/repo'
  // decide whether to use tag: or ref: based on whether it's a SHA
  const isSha = !suggestionVersion.includes('.') && suggestionVersion.length >= 7 && /^[0-9a-f]+$/i.test(suggestionVersion);
  const newOption = isSha ? 'ref' : 'tag';

  return `${suggestionUpdate.parsedVersion}, ${newOption}: '${suggestionVersion}'`;
}
