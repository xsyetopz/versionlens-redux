import type { PackageResponse, TSuggestionReplaceFunction } from '#domain/packages';
import { type TextDocument, CodeLens, Range, Uri } from 'vscode';

export class SuggestionCodeLens extends CodeLens {

  constructor(
    commandRange: Range,
    readonly replaceRange: Range,
    readonly packageResponse: PackageResponse,
    readonly documentUrl: Uri,
    readonly replaceVersionFn: TSuggestionReplaceFunction
  ) {
    super(commandRange);
    this.replaceRange = replaceRange ?? commandRange;
    // this.package = packageResponse;
    this.command = undefined;
  }

  setCommand(title: string, command: string, args?: Array<any>) {
    this.command = {
      title,
      command,
      arguments: args
    };
    return this;
  }

  static createFromPackageResponses(
    document: TextDocument,
    suggestions: Array<PackageResponse>,
    replaceVersionFn: TSuggestionReplaceFunction,
  ): Array<SuggestionCodeLens> {
    return suggestions.map(
      function (response) {
        return SuggestionCodeLens.createFromPackageResponse(
          response,
          document,
          replaceVersionFn
        );
      }
    );
  }

  static createFromPackageResponse(
    packageResponse: PackageResponse,
    document: TextDocument,
    replaceVersionFn: TSuggestionReplaceFunction,
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

}