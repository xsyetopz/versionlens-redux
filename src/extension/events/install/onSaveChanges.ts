import type { ILogger } from '#domain/logging';
import { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { IDisposable } from '#domain/utils';
import type { IVersionLensState } from '#extension';
import type { IVsCodeTasks } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { Task } from 'vscode';

export class OnSaveChanges {

  static log = {
    skipSaveChangesTask: 'Skipping "{providerName}.onSaveChanges" because a custom task was not provided.',
    saveChangesTaskNotFound: 'Could not find the {providerName}.onSaveChanges["{onSaveChangesTask}"] task.',
    executingTask: 'Executing {providerName}.onSaveChanges["{onSaveChangesTask}"] task.',
    taskCompleted: '{providerName}.onSaveChanges["{onSaveChangesTask}"] task exited with {exitCode}.'
  } as const

  constructor(
    readonly fileWatcherDependencyCache: DependencyCache,
    readonly editorDependencyCache: DependencyCache,
    readonly tasks: IVsCodeTasks,
    readonly state: IVersionLensState,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('fileWatcherDependencyCache', fileWatcherDependencyCache);
    throwUndefinedOrNull('editorDependencyCache', editorDependencyCache);
    throwUndefinedOrNull("tasks", tasks);
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(provider: ISuggestionProvider, packageFilePath: string): Promise<void> {
    // update the file watcher dependencies
    const deps = this.editorDependencyCache.get(provider.name, packageFilePath) ?? [];
    this.fileWatcherDependencyCache.set(provider.name, packageFilePath, deps)

    // remove the packageFilePath from editor dependency cache
    this.editorDependencyCache.remove(provider.name, packageFilePath);
    this.logger.debug(
      "cleared editor dependency cache for {packageFilePath}",
      packageFilePath
    );

    // check we have a task to run
    if (!provider.config.onSaveChangesTask) {
      this.logger.info(OnSaveChanges.log.skipSaveChangesTask, provider.name);
      return;
    }

    // fetch the custom task for the provider
    const availableTasks = await this.tasks.fetchTasks();
    const filteredTasks = availableTasks.filter(
      x => x.name == provider.config.onSaveChangesTask
    );

    // check we found a task
    if (filteredTasks.length == 0) {
      this.logger.error(
        OnSaveChanges.log.saveChangesTaskNotFound,
        provider.name,
        provider.config.onSaveChangesTask
      );
      return;
    }

    this.logger.info(
      OnSaveChanges.log.executingTask,
      provider.name,
      provider.config.onSaveChangesTask
    );

    // execute the task
    const exitCode = await this.executeTask(filteredTasks[0])

    this.logger.info(
      OnSaveChanges.log.taskCompleted,
      provider.name,
      provider.config.onSaveChangesTask,
      exitCode
    );

    // reset outdated flag
    if (exitCode === 0) await this.state.showOutdated.change(false);
  }

  private async executeTask(task: Task): Promise<number | undefined> {
    await this.tasks.executeTask(task);
    return new Promise((resolve, reject) => {
      const disposables: IDisposable[] = []
      disposables.push(
        this.tasks.onDidEndTaskProcess(
          e => {
            if (task.name === e.execution.task.name)
              resolve(e.exitCode);
          },
          this,
          disposables
        )
      )
    });
  }

}