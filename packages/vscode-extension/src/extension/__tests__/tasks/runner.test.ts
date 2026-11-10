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
let endListeners: ((event: {
	execution: ExecutionStub | undefined;
	exitCode: number | undefined;
}) => void)[] = [];

mockVscodeHost((): MockModule => ({
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
				execution: ExecutionStub | undefined;
				exitCode: number | undefined;
			}) => void,
		): { dispose: () => void } {
			endListeners.push(listener);
			return {
				dispose(): void {
					endListeners = endListeners.filter(
						(entry): boolean => entry !== listener,
					);
				},
			};
		},
	},
	window: { showWarningMessage: (): undefined => undefined },
}));

it("runTask selects the document workspace and ignores same-name executions", async (): Promise<void> => {
	const { runTask } = await import("../../tasks/runner.ts");
	availableTasks.length = 0;
	executed.length = 0;
	endListeners = [];
	const folderA = workspaceFolder("file:///workspace-a");
	const folderB = workspaceFolder("file:///workspace-b");
	const taskA = { name: "install", scope: folderA };
	const taskB = { name: "install", scope: folderB };
	availableTasks.push(taskA, taskB);

	let completed = false;
	const pending = (async (): Promise<TaskRunResult> => {
		const result = await runTask("install", folderB);
		completed = true;
		return result;
	})();
	await Promise.resolve();
	expect(executed[0]?.task).toBe(taskB);

	finish({ task: taskA, terminate: (): undefined => undefined }, 0);
	await Promise.resolve();
	expect(completed).toBe(false);

	const [selectedExecution] = executed;
	finish(selectedExecution, failedTaskExitCode);
	expect(await pending).toEqual({
		exitCode: failedTaskExitCode,
		kind: "completed",
	});
});

const workspaceFolder = (uri: string): { uri: { toString: () => string } } => ({
	uri: { toString: (): string => uri },
});

const finish = (
	execution: ExecutionStub | undefined,
	exitCode: number | undefined,
): void => {
	for (const listener of endListeners) {
		listener({ execution, exitCode });
	}
};
