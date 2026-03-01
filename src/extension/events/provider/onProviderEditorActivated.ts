import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import type { VersionLensExtension } from '#extension';
import type { PackageFileWatcher } from '#extension/watcher';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';
import type { TextDocument } from 'vscode';

/**
 * Event handler for when a text editor with a supported package provider is activated.
 */
export class OnProviderEditorActivated {

  /**
   * Initializes a new instance of the OnProviderEditorActivated class.
   * @param extension The extension instance.
   * @param packageFileWatcher The package file watcher.
   * @param logger Logger instance.
   */
  constructor(
    readonly extension: VersionLensExtension,
    readonly packageFileWatcher: PackageFileWatcher,
    readonly logger: ILogger,
  ) {
    throwUndefinedOrNull("extension", extension);
    throwUndefinedOrNull("packageFileWatcher", packageFileWatcher);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes when a provider-supported document is focused.
   * Ensures the file is being watched, even if it's outside the workspace.
   * @param activeProvider The provider associated with the document.
   * @param document The VS Code text document.
   */
  async execute(activeProvider: ISuggestionProvider, document: TextDocument): Promise<void> {
    this.logger.debug("{providerName} provider editor activated", activeProvider.name);

    // get the package file path
    const packageFilePath = document.uri.fsPath;
    const packagePath = dirname(packageFilePath);

    // check if the file is in the workspace
    const packageFileInWorkspace = packagePath.startsWith(this.extension.projectPath);
    if (packageFileInWorkspace === false) {
      // add the outside package file to the watcher
      await this.packageFileWatcher.watchFile(document.uri);
    }
  }

}