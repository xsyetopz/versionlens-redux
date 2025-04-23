import type { PackageResponse, SuggestionReplaceFunction } from '#domain/packages';
import type { ISuggestionCodeLens } from '#extension';
import { type Range, type Uri, CodeLens } from 'vscode';

export class SuggestionCodeLens extends CodeLens implements ISuggestionCodeLens {

  constructor(
    commandRange: Range,
    readonly replaceRange: Range,
    readonly packageResponse: PackageResponse,
    readonly documentUrl: Uri,
    readonly replaceVersionFn: SuggestionReplaceFunction
  ) {
    super(commandRange);
    this.replaceRange = replaceRange ?? commandRange;
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

}