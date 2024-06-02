import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import {
  DependencyCache,
  IPackageFileWatcher,
  OnPackageDependenciesChangedEvent
} from 'domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { DependencyChangesResult, GetDependencyChanges } from 'domain/useCases';
import { AsyncEmitter, IDisposable } from 'domain/utils';
import { isMatch } from 'micromatch';
import { Uri } from 'vscode';
import { IWorkspaceAdapter } from '.';

export class PackageFileWatcher
  extends AsyncEmitter<OnPackageDependenciesChangedEvent>
  implements IPackageFileWatcher, IDisposable {

  constructor(
    readonly getDependencyChanges: GetDependencyChanges,
    readonly workspace: IWorkspaceAdapter,
    readonly providers: ISuggestionProvider[],
    readonly dependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("getDependencyChanges", getDependencyChanges);
    throwUndefinedOrNull("workspace", workspace);
    throwUndefinedOrNull("providers", providers);
    throwUndefinedOrNull("dependencyCache", dependencyCache);
    throwUndefinedOrNull("logger", logger);

    this.disposables = [];
  }

  private disposables: IDisposable[];

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
      'initialized PackageFileWatcher (%s ms)',
      Math.floor(completedAt - startedAt)
    );

    this.watch();
  }

  async watchFile(file: Uri): Promise<void> {
    const matched = this.providers.filter(
      provider => isMatch(file.fsPath, provider.config.fileMatcher.pattern, { dot: true })
    );

    if (matched.length === 0) {
      this.logger.error(
        `could not find '%s' project file`,
        file.fsPath
      );
      return;
    }

    const provider = matched[0];
    await this.onFileAdd(provider, file);
    this.logger.debug(
      'found %s project file for %s',
      1,
      provider.name
    );

    this.watch();
  }

  watch(): void {
    // watch files
    this.providers.forEach(provider => {
      const watcher = this.workspace.createFileSystemWatcher(
        provider.config.fileMatcher.pattern
      );

      this.logger.debug(
        `created watcher for '%s' with pattern '%s'`,
        provider.name,
        provider.config.fileMatcher.pattern
      );

      this.disposables.push(
        watcher.onDidCreate(this.onFileCreate.bind(this, provider)),
        watcher.onDidDelete(this.onFileDelete.bind(this, provider)),
        watcher.onDidChange(this.onFileChange.bind(this, provider)),
        watcher
      );
    });
  }

  async dispose(): Promise<void> {
    this.disposables.forEach(async disposable => await disposable.dispose());
  }

  async onFileAdd(provider: ISuggestionProvider, uri: Uri) {
    this.logger.silly("file added '%s'", uri);
    await this.updateCacheFromFile(provider, uri.fsPath);
  }

  private async onFileCreate(provider: ISuggestionProvider, uri: Uri) {
    this.logger.silly("file created '%s'", uri);
    await this.updateCacheFromFile(provider, uri.fsPath);
  }

  private onFileDelete(provider: ISuggestionProvider, uri: Uri) {
    this.logger.silly("file removed '%s'", uri);
    this.dependencyCache.remove(provider.name, uri.fsPath);
  }

  async onFileChange(provider: ISuggestionProvider, uri: Uri) {
    this.logger.silly("file changed '%s'", uri);

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
    this.logger.silly("updating package dependency cache for '%s'", packageFilePath);
    this.dependencyCache.set(provider.name, packageFilePath, result.parsedDependencies);

    return result;
  }

  private async findProviderFiles(provider: ISuggestionProvider) {
    // capture start time
    const startedAt = performance.now();

    const files = await this.workspace.findFiles(
      provider.config.fileMatcher.pattern,
      provider.config.fileMatcher.exclude
    );

    for (const file of files) {
      await this.onFileAdd(provider, file)
    }

    // report completed duration
    const completedAt = performance.now();
    this.logger.debug(
      'found %s project files for %s (%s ms)',
      files.length,
      provider.name,
      Math.floor(completedAt - startedAt)
    );
  }

}