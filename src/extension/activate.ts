import type { IDomainServices } from '#domain';
import type { IServiceProvider } from '#domain/di';
import type { ILogger, ILoggingOptions } from '#domain/logging';
import type { IPackageFileWatcher } from '#domain/packages';
import { nameOf, readJsonFile } from '#domain/utils';
import {
  type IExtensionServices,
  OnActiveTextEditorChange,
  VersionLensExtension
} from '#extension';
import { dirname, join } from 'node:path';
import { type ExtensionContext, window, workspace } from 'vscode';
import { configureContainer } from './extensionContainer';

let serviceProvider: IServiceProvider;

export async function activate(context: ExtensionContext): Promise<void> {
  // get the resource folder path (opened folder or single file)
  const resourceFolderPath = await getResourceFolderPath(context);

  // create the ioc service provider
  serviceProvider = await configureContainer(context, resourceFolderPath);

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
  logger.info("resource folder path: %s", resourceFolderPath);
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
    // auth events
    serviceNames.onAddUrlAuthentication,
    serviceNames.onRemoveUrlAuthentication,
    // commands events
    serviceNames.onClearCache,
    serviceNames.onFileLinkClick,
    serviceNames.onUpdateDependencyClick,
    // editorTitleBar events
    serviceNames.onErrorClick,
    serviceNames.onToggleReleases,
    serviceNames.onTogglePrereleases,
    // install events
    serviceNames.onPreSaveChanges, // will instantiate onTextDocumentSave
    serviceNames.onSaveChanges,
    // provider document events
    serviceNames.onProviderEditorActivated, // will instantiate onActiveTextEditorChange
    serviceNames.onProviderTextDocumentChange, // will instantiate onTextDocumentChange
    serviceNames.onProviderTextDocumentClose, // will instantiate onTextDocumentClose
    // watcher events
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

async function getResourceFolderPath(context: ExtensionContext): Promise<string> {
  if (context.storageUri) return context.storageUri.path;
  const resourceFilePath = window.activeTextEditor.document.uri.path;
  return dirname(resourceFilePath);
}