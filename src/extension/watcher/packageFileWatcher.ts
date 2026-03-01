import type { ILogger } from '#domain/logging';
import type { DependencyCache, OnPackageDependenciesChangedEvent } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { DependencyChangesResult, GetDependencyChanges } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import type { EditorConfig, IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { isMatch } from 'micromatch';
import type { Uri } from 'vscode';

/**
 * Default glob patterns to exclude from file watching.
 */
export const defaultExcludes = [
  '**/node_modules/**',
  '**/bower_components/**',
  '**/bin/**',
  '**/.git/**',
  '**/.vscode/**'
];

/**
 * Watches the workspace for changes to package files and maintains the dependency cache.
 */
export class PackageFileWatcher extends AsyncEmitter<OnPackageDependenciesChangedEvent> {

  /**
   * Initializes a new instance of the PackageFileWatcher class.
   * @param getDependencyChanges Use case for detecting dependency changes.
   * @param providers List of active suggestion providers.
   * @param dependencyCache Cache for storing parsed dependencies.
   * @param editorConfig VS Code editor configuration.
   * @param workspace VS Code workspace adapter.
   * @param logger Logger instance.
   */
  constructor(
    readonly getDependencyChanges: GetDependencyChanges,
    readonly providers: ISuggestionProvider[],
    readonly dependencyCache: DependencyCache,
    readonly editorConfig: EditorConfig,
    readonly workspace: IVsCodeWorkspace,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("getDependencyChanges", getDependencyChanges);
    throwUndefinedOrNull("workspace", workspace);
    throwUndefinedOrNull("providers", providers);
    throwUndefinedOrNull("dependencyCache", dependencyCache);
    throwUndefinedOrNull("editorConfig", editorConfig);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Scans the workspace folders for supported package files and starts watching them.
   * @returns A promise that resolves when the initial scan is complete.
   */
  async watchFolder(): Promise<void> {
    const startedAt = performance.now();

    // queue promises
    const promises = [];
    for (const provider of this.providers) {
      promises.push(this.findProviderFiles(provider));
    }

    // parallel promises
    await Promise.all(promises);

    const completedAt = performance.now();
    this.logger.debug(
      'initialized PackageFileWatcher ({duration} ms)',
      Math.floor(completedAt - startedAt)
    );

    this.watch();
  }

  /**
   * Adds a specific file to the watch list.
   * @param file The URI of the file to watch.
   * @returns A promise that resolves when the file has been processed.
   */
  async watchFile(file: Uri): Promise<void> {
    const matched = this.providers.filter(
      provider => isMatch(file.fsPath, provider.config.filePatterns, { dot: true })
    );

    if (matched.length === 0) {
      this.logger.error(
        `could not find '{filePath}' project file`,
        file.fsPath
      );
      return;
    }

    const provider = matched[0];
    await this.onFileAdd(provider, file);
    this.logger.debug(
      'found 1 project file for {providerName}',
      provider.name
    );

    this.watch();
  }

  /**
   * Sets up VS Code file system watchers for all active providers.
   */
  watch(): void {
    // watch files
    this.providers.forEach(provider => {
      const watcher = this.workspace.createFileSystemWatcher(provider.config.filePatterns);

      this.logger.debug(
        "created watcher for '{providerName}' with pattern '{filePatterns}'",
        provider.name,
        provider.config.filePatterns
      );

      this.disposables.push(
        watcher.onDidCreate(this.onFileCreate.bind(this, provider)),
        watcher.onDidDelete(this.onFileDelete.bind(this, provider)),
        watcher.onDidChange(this.onFileChange.bind(this, provider)),
        watcher
      );
    });
  }

  /**
   * Handles a file being added to the watch list.
   * @param provider The provider associated with the file.
   * @param uri The URI of the added file.
   */
  async onFileAdd(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file added '{uri}'", uri.toString());
    await this.updateCacheFromFile(provider, uri.fsPath);
  }

  /**
   * Handles a file creation event.
   */
  private async onFileCreate(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file created '{uri}'", uri.toString());
    await this.updateCacheFromFile(provider, uri.fsPath);
  }

  /**
   * Handles a file deletion event.
   */
  private onFileDelete(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file removed '{uri}'", uri.toString());
    this.dependencyCache.remove(provider.name, uri.fsPath);
  }

  /**
   * Handles a file change event.
   * Fires the dependency changed event if changes were detected.
   * @param provider The provider associated with the file.
   * @param uri The URI of the changed file.
   */
  async onFileChange(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file changed '{uri}'", uri.toString());

    const packageFilePath = uri.fsPath;
    const result = await this.updateCacheFromFile(provider, packageFilePath);

    // notify dependencies updated to listener
    if (result.hasChanged) {
      await this.fire(
        provider,
        packageFilePath,
        result.parsedDependencies
      );
    }
  }

  /**
   * Parses dependencies from a file and updates the cache.
   */
  private async updateCacheFromFile(
    provider: ISuggestionProvider,
    packageFilePath: string
  ): Promise<DependencyChangesResult> {

    const result = await this.getDependencyChanges.execute(provider, packageFilePath);
    this.logger.trace(
      "updating package dependency cache for '{packageFilePath}'",
      packageFilePath
    );
    this.dependencyCache.set(provider.name, packageFilePath, result.parsedDependencies);

    return result;
  }

  /**
   * Searches for project files matching a provider's configuration.
   */
  private async findProviderFiles(provider: ISuggestionProvider) {
    // capture start time
    const startedAt = performance.now();
    const { filePatterns, fileExcludePatterns } = provider.config;
    const excludeFiles = this.editorConfig.excludeFiles;
    const excludePatterns = [
      ...defaultExcludes,
      ...Object.keys(excludeFiles).filter(x => excludeFiles[x])
    ];

    if (fileExcludePatterns) excludePatterns.push(...fileExcludePatterns);

    const files = await this.workspace.findFiles(
      filePatterns,
      mapToSinglePattern(excludePatterns)
    );

    for (const file of files) {
      await this.onFileAdd(provider, file)
    }

    // report completed duration
    const completedAt = performance.now();
    this.logger.debug(
      'found {fileCount} project files for {providerName} ({duration} ms)',
      files.length,
      provider.name,
      Math.floor(completedAt - startedAt)
    );
  }
}

/**
 * Combines multiple glob patterns into a single brace-enclosed pattern.
 * @param patterns The list of glob patterns.
 * @returns A single glob pattern string.
 */
export function mapToSinglePattern(patterns: string[]): string {
  return `{${patterns.join(',')}}`;
}