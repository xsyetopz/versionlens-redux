import type { PackageResponse, SuggestionReplaceFunction } from '#domain/packages';
import type { ISuggestionCodeLens } from '#extension';
import { type Range, type Uri, CodeLens } from 'vscode';

/**
 * Represents a VS Code CodeLens tailored for package version suggestions.
 */
export class SuggestionCodeLens extends CodeLens implements ISuggestionCodeLens {

  /**
   * Initializes a new instance of the SuggestionCodeLens class.
   * @param commandRange The range where the code lens command will be displayed.
   * @param replaceRange The range in the document that will be replaced when the suggestion is applied.
   * @param packageResponse The underlying package suggestion data.
   * @param documentUrl The URI of the document containing this code lens.
   * @param replaceVersionFn The function used to generate the replacement version string.
   */
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

  /**
   * Sets the VS Code command for this code lens.
   * @param title The display title of the command.
   * @param command The identifier of the command to execute.
   * @param args Arguments to pass to the command.
   * @returns The current instance for chaining.
   */
  setCommand(title: string, command: string, args?: Array<any>) {
    this.command = {
      title,
      command,
      arguments: args
    };
    return this;
  }

}