import type { ILogger, ILoggerChannel } from '#domain/logging';
import type { IPackageFileWatcher } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { VersionLensExtension } from '#extension';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';
import type { TextDocument } from 'vscode';

export class OnProviderEditorActivated {

  constructor(
    readonly loggerChannel: ILoggerChannel,
    readonly extension: VersionLensExtension,
    readonly packageFileWatcher: IPackageFileWatcher,
    readonly logger: ILogger,
  ) {
    throwUndefinedOrNull("loggerChannel", loggerChannel);
    throwUndefinedOrNull("extension", VersionLensExtension);
    throwUndefinedOrNull("packageFileWatcher", packageFileWatcher);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(activeProvider: ISuggestionProvider, document: TextDocument): Promise<void> {
    this.logger.debug("%s provider editor activated", activeProvider.name);

    // ensure the latest logging level is set
    this.loggerChannel.refreshLoggingLevel();

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