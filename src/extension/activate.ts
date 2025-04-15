import type { IDomainServices } from '#domain';
import type { IServiceProvider } from '#domain/di';
import { LogLevel, type LoggerFactory } from '#domain/logging';
import { nameOf } from '#domain/utils';
import {
  type IExtensionServices,
  OnActiveTextEditorChange,
  VersionLensExtension
} from '#extension';
import type { EditorConfig } from '#extension/vscode';
import { dirname, join } from 'node:path';
import { type ExtensionContext, LogOutputChannel, window } from 'vscode';
import { configureContainer } from './extensionContainer';
import type { PackageFileWatcher } from './watcher';

let serviceProvider: IServiceProvider;

export async function activate(context: ExtensionContext): Promise<void> {
  // get the resource folder path (opened folder or single file)
  const resourceFolderPath = await getResourceFolderPath(context);

  // create the ioc service provider
  serviceProvider = await configureContainer(context, resourceFolderPath);

  const serviceNames = nameOf<IDomainServices & IExtensionServices>();

  // get the logger
  const loggerFactory = serviceProvider.getService<LoggerFactory>(serviceNames.loggerFactory);
  const logger = loggerFactory.create('activate');
  const logOutputChannel = serviceProvider.getService<LogOutputChannel>(
    serviceNames.logOutputChannel
  );

  // get the editorConfig
  const editorConfig = serviceProvider.getService<EditorConfig>(
    serviceNames.editorConfig
  );

  // check editor.codeLens is enabled
  if (editorConfig.codeLens === false) {
    logger.error(
      "Code lenses are disabled. This extension won't work unless you enable 'editor.codeLens' in your vscode settings"
    );
  }

  // get the extension info
  const extension = serviceProvider.getService<VersionLensExtension>(
    serviceNames.extension
  );
  const extensionPath = context.asAbsolutePath("");

  // log general start up info
  logger.info("extension path: {extensionPath}", extensionPath);
  logger.info("resource folder path: {resourceFolderPath}", join(resourceFolderPath, ".."));
  logger.info("workspace mode: {isWorkspaceMode}", extension.isWorkspaceMode);
  logger.info("version: {version}", context.extension.packageJSON.version);
  logger.info("log level: {logLevel}", LogLevel[logOutputChannel.logLevel]);
  logger.info("log folder: {logPath}", join(context.logUri.fsPath, ".."));

  // setup package dependency watcher
  const watcher = serviceProvider.getService<PackageFileWatcher>(
    serviceNames.packageFileWatcher
  );

  if (extension.isWorkspaceMode)
    // watch workspace project files
    await watcher.watchFolder();
  else
    // watch single project file
    await watcher.watchFile(window.activeTextEditor!.document.uri)

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
  const resourceFilePath = window.activeTextEditor!.document.uri.path;
  return dirname(resourceFilePath);
}