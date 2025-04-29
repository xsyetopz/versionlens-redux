import type { ILogger } from '#domain/logging';
import type { DependencyCache, OnPackageDependenciesChangedEvent } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { DependencyChangesResult, GetDependencyChanges } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import type { EditorConfig, IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { isMatch } from 'micromatch';
import type { Uri } from 'vscode';

export const defaultExcludes = [
  '**/node_modules/**',
  '**/bower_components/**',
  '**/.git/**',
  '**/.vscode/**'
];

export class PackageFileWatcher extends AsyncEmitter<OnPackageDependenciesChangedEvent> {

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

  async onFileAdd(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file added '{uri}'", uri.toString());
    await this.updateCacheFromFile(provider, uri.fsPath);
  }

  private async onFileCreate(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file created '{uri}'", uri.toString());
    await this.updateCacheFromFile(provider, uri.fsPath);
  }

  private onFileDelete(provider: ISuggestionProvider, uri: Uri) {
    const matched = isMatch(uri.fsPath, defaultExcludes, { dot: true })
    if (matched) return;

    this.logger.trace("file removed '{uri}'", uri.toString());
    this.dependencyCache.remove(provider.name, uri.fsPath);
  }

  async onFileChange(provider: ISuggestionProvider, uri: Uri) {
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

export function mapToSinglePattern(patterns: string[]): string {
  return `{${patterns.join(',')}}`;
}