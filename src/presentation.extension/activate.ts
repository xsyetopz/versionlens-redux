import { IServiceProvider } from '#domain/di';
import { ILogger, ILoggingOptions } from '#domain/logging';
import { IPackageFileWatcher } from '#domain/packages';
import { IDomainServices } from '#domain/services';
import { nameOf, readJsonFile } from '#domain/utils';
import {
  configureContainer,
  IExtensionServices,
  OnActiveTextEditorChange,
  VersionLensExtension
} from '#extension';
import { join } from 'node:path';
import { ExtensionContext, window, workspace } from 'vscode';

let serviceProvider: IServiceProvider;

export async function activate(context: ExtensionContext): Promise<void> {
  // create the ioc service provider
  serviceProvider = await configureContainer(context)

  const serviceNames = nameOf<IDomainServices & IExtensionServices>();

  // get the logger
  const logger = serviceProvider.getService<ILogger>(serviceNames.logger);
  const loggingOptions = serviceProvider.getService<ILoggingOptions>(
    serviceNames.loggingOptions
  );

  // check editor.codeLens is enabled
  const codeLensEnabled = workspace.getConfiguration().get('editor.codeLens')
  if (codeLensEnabled === false) {
    logger.error(
      "Code lenses are disabled. This extension won't work unless you enable 'editor.codeLens' in your vscode settings"
    );
  }

  // get the extension info
  const extension = serviceProvider.getService<VersionLensExtension>(
    serviceNames.extension
  );
  const extensionPath = context.asAbsolutePath("");
  const packageJsonPath = context.asAbsolutePath("package.json");
  const { version } = await readJsonFile<any>(packageJsonPath);

  // log general start up info
  const logPath = join(context.logUri.fsPath, "..");
  logger.info("extension path: %s", extensionPath);
  logger.info("workspace mode: %s", extension.isWorkspaceMode);
  logger.info("version: %s", version);
  logger.info("log level: %s", loggingOptions.level);
  logger.info("log folder: %s", logPath);

  // setup package dependency watcher
  const watcher = serviceProvider.getService<IPackageFileWatcher>(
    serviceNames.packageFileWatcher
  );

  if (extension.isWorkspaceMode)
    // watch workspace project files
    await watcher.watchFolder();
  else
    // watch single project file
    await watcher.watchFile(window.activeTextEditor.document.uri)

  // instantiate dependencies that aren't referenced
  const instantiateDeps = [
    // commands
    serviceNames.onClearCache,
    serviceNames.onFileLinkClick,
    serviceNames.onUpdateDependencyClick,
    // editorTitleBar
    serviceNames.onErrorClick,
    serviceNames.onToggleReleases,
    serviceNames.onTogglePrereleases,
    // install
    serviceNames.onPreSaveChanges, // will instantiate onTextDocumentSave
    serviceNames.onSaveChanges,
    // provider documents
    serviceNames.onProviderEditorActivated, // will instantiate onActiveTextEditorChange
    serviceNames.onProviderTextDocumentChange, // will instantiate onTextDocumentChange
    serviceNames.onProviderTextDocumentClose, // will instantiate onTextDocumentClose
    // watcher
    serviceNames.onPackageDependenciesChanged
  ];

  instantiateDeps.forEach(x => serviceProvider.getService(x));

  // ensure this is run when the extension is first loaded
  serviceProvider.getService<OnActiveTextEditorChange>(serviceNames.onActiveTextEditorChange)
    .execute(window.activeTextEditor)
}

export async function deactivate() {
  await serviceProvider.dispose();
}