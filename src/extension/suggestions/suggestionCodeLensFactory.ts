import { type PackageResponse, SuggestionReplaceFunction } from '#domain/packages';
import { SuggestionCodeLens } from '#extension/suggestions';
import { type TextDocument, Range, Uri } from 'vscode';

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