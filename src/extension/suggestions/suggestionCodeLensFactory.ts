import { type PackageResponse, SuggestionReplaceFunction } from '#domain/packages';
import { SuggestionCodeLens } from '#extension/suggestions';
import { type TextDocument, Range, Uri } from 'vscode';

/**
 * Creates an array of SuggestionCodeLens objects from a list of package responses.
 * @param document The VS Code text document.
 * @param suggestions The list of package version suggestions.
 * @param replaceVersionFn The function used to generate replacement version strings.
 * @returns An array of initialized code lenses.
 */
export function createFromPackageResponses(
  document: TextDocument,
  suggestions: Array<PackageResponse>,
  replaceVersionFn: SuggestionReplaceFunction,
): Array<SuggestionCodeLens> {
  return suggestions.map(
    function (response) {
      return createFromPackageResponse(
        response,
        document,
        replaceVersionFn
      );
    }
  );
}

/**
 * Creates a single SuggestionCodeLens from a package response.
 * Calculates the appropriate document ranges for display and replacement.
 */
function createFromPackageResponse(
  packageResponse: PackageResponse,
  document: TextDocument,
  replaceVersionFn: SuggestionReplaceFunction,
): SuggestionCodeLens {
  const { nameRange, versionRange } = packageResponse.parsedDependency;
  const commandRangePos = nameRange.start + packageResponse.order;
  const commandRange = new Range(
    document.positionAt(commandRangePos),
    document.positionAt(commandRangePos)
  );
  const replaceRange = new Range(
    document.positionAt(versionRange.start),
    document.positionAt(versionRange.end)
  );
  return new SuggestionCodeLens(
    commandRange,
    replaceRange,
    packageResponse,
    Uri.file(document.fileName),
    replaceVersionFn
  );
}