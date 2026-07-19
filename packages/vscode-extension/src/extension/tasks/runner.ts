import {
  type Disposable,
  type Task,
  type TaskExecution,
  tasks,
  type WorkspaceFolder,
  window,
} from "#vscode-host";

const activeTasks = new Set<string>();

type TaskRunResult =
  | { exitCode: number | undefined; kind: "completed" }
  | { kind: "busy" }
  | { kind: "notFound" };

async function runTask(
  label: string,
  workspaceFolder?: WorkspaceFolder,
): Promise<TaskRunResult> {
  const activeKey = `${workspaceFolder?.uri.toString() ?? "<workspace>"}\0${label}`;
  if (activeTasks.has(activeKey)) {
    return { kind: "busy" };
  }

  const namedTasks = (await tasks.fetchTasks()).filter(
    (item): boolean => item.name === label,
  );
  const task = selectWorkspaceTask(namedTasks, workspaceFolder);
  if (!task) {
    window.showWarningMessage(`Version Lens task not found: ${label}`);
    return { kind: "notFound" };
  }

  activeTasks.add(activeKey);
  let disposable: Disposable | undefined;
  try {
    let execution: TaskExecution | undefined;
    const earlyCompletions = new Map<TaskExecution, number | undefined>();
    let complete: ((result: TaskRunResult) => void) | undefined;
    const completed = new Promise<TaskRunResult>((resolve): void => {
      complete = resolve;
      disposable = tasks.onDidEndTaskProcess((event): void => {
        if (!execution) {
          earlyCompletions.set(event.execution, event.exitCode);
          return;
        }
        if (event.execution !== execution) {
          return;
        }

        disposable?.dispose();
        activeTasks.delete(activeKey);
        resolve({ exitCode: event.exitCode, kind: "completed" });
      });
    });

    execution = await tasks.executeTask(task);
    if (earlyCompletions.has(execution)) {
      const exitCode = earlyCompletions.get(execution);
      disposable?.dispose();
      activeTasks.delete(activeKey);
      complete?.({ exitCode, kind: "completed" });
    }
    return await completed;
  } catch (error) {
    disposable?.dispose();
    activeTasks.delete(activeKey);
    throw error;
  }
}

function selectWorkspaceTask(
  availableTasks: Task[],
  workspaceFolder: WorkspaceFolder | undefined,
): Task | undefined {
  if (!workspaceFolder) {
    return availableTasks.find(
      (task): boolean => typeof task.scope !== "object",
    );
  }

  const workspaceUri = workspaceFolder.uri.toString();
  return (
    availableTasks.find(
      (task): boolean =>
        typeof task.scope === "object" &&
        task.scope?.uri.toString() === workspaceUri,
    ) ?? availableTasks.find((task): boolean => typeof task.scope !== "object")
  );
}

export type { TaskRunResult };
export { runTask };
