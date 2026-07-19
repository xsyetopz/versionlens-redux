import type { TaskRunResult } from "../../tasks/runner.ts";
import { expect, it, mockVscodeHost } from "../runtime.ts";

type MockModule = Record<string, unknown>;

interface TaskStub {
  name: string;
  scope?: { uri: { toString: () => string } };
}

interface ExecutionStub {
  task: TaskStub;
  terminate: () => void;
}

const failedTaskExitCode = 7;

const availableTasks: TaskStub[] = [];
const executed: ExecutionStub[] = [];
const endListeners: ((event: {
  execution: ExecutionStub;
  exitCode: number | undefined;
}) => void)[] = [];

mockVscodeHost(
  (): MockModule => ({
    tasks: {
      executeTask(task: TaskStub): {
        task: TaskStub;
        terminate: () => undefined;
      } {
        const execution = { task, terminate: (): undefined => undefined };
        executed.push(execution);
        return execution;
      },
      fetchTasks: (): TaskStub[] => availableTasks,
      onDidEndTaskProcess(
        listener: (event: {
          execution: ExecutionStub;
          exitCode: number | undefined;
        }) => void,
      ): { dispose: () => void } {
        endListeners.push(listener);
        return {
          dispose(): void {
            const index = endListeners.indexOf(listener);
            if (index >= 0) {
              endListeners.splice(index, 1);
            }
          },
        };
      },
    },
    window: { showWarningMessage: (): undefined => undefined },
  }),
);

it("runTask selects the document workspace and ignores same-name executions", async (): Promise<void> => {
  const { runTask } = await import("../../tasks/runner.ts");
  availableTasks.length = 0;
  executed.length = 0;
  endListeners.length = 0;
  const folderA = workspaceFolder("file:///workspace-a");
  const folderB = workspaceFolder("file:///workspace-b");
  const taskA = { name: "install", scope: folderA };
  const taskB = { name: "install", scope: folderB };
  availableTasks.push(taskA, taskB);

  let completed = false;
  const pending = runTask("install", folderB as never).then(
    (result): TaskRunResult => {
      completed = true;
      return result;
    },
  );
  await Promise.resolve();
  expect(executed[0]?.task).toBe(taskB);

  finish({ task: taskA, terminate: (): undefined => undefined }, 0);
  await Promise.resolve();
  expect(completed).toBe(false);

  const [selectedExecution] = executed;
  if (!selectedExecution) {
    throw new Error("expected selected task execution");
  }
  finish(selectedExecution, failedTaskExitCode);
  expect(await pending).toEqual({
    exitCode: failedTaskExitCode,
    kind: "completed",
  });
});

function workspaceFolder(uri: string): { uri: { toString: () => string } } {
  return { uri: { toString: (): string => uri } };
}

function finish(execution: ExecutionStub, exitCode: number | undefined): void {
  for (const listener of [...endListeners]) {
    listener({ execution, exitCode });
  }
}
