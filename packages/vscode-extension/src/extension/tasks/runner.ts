import {
	type Disposable,
	type Task,
	type TaskExecution,
	tasks,
	window,
} from "#vscode-host";

const activeTasks = new Set<string>();

/** The part of a `WorkspaceFolder` that task selection reads. */
interface TaskFolder {
	uri: { toString: () => string };
}

type TaskRunResult =
	| { exitCode: number | undefined; kind: "completed" }
	| { kind: "busy" }
	| { kind: "notFound" };

const runTask = async (
	label: string,
	workspaceFolder?: TaskFolder,
): Promise<TaskRunResult> => {
	const activeKey = `${workspaceFolder?.uri.toString() ?? "<workspace>"}\0${label}`;
	if (activeTasks.has(activeKey)) {
		return { kind: "busy" };
	}

	const fetched = await tasks.fetchTasks();
	const namedTasks = fetched.filter((item): boolean => item.name === label);
	const task = selectWorkspaceTask(namedTasks, workspaceFolder);
	if (!task) {
		window.showWarningMessage(`Version Lens task not found: ${label}`);
		return { kind: "notFound" };
	}

	activeTasks.add(activeKey);
	return executeAndAwait(task, activeKey);
};

const executeAndAwait = async (
	task: Task,
	activeKey: string,
): Promise<TaskRunResult> => {
	let disposable: Disposable | undefined;
	try {
		// A holder, because the listener below runs before `executeTask` resolves.
		const started: { execution?: TaskExecution } = {};
		const earlyCompletions = new Map<TaskExecution, number | undefined>();
		let complete: ((result: TaskRunResult) => void) | undefined;
		const completed = new Promise<TaskRunResult>((resolve): void => {
			complete = resolve;
			disposable = tasks.onDidEndTaskProcess((event): void => {
				if (!started.execution) {
					earlyCompletions.set(event.execution, event.exitCode);
					return;
				}
				if (event.execution !== started.execution) {
					return;
				}

				disposable?.dispose();
				activeTasks.delete(activeKey);
				resolve({ exitCode: event.exitCode, kind: "completed" });
			});
		});

		const execution = await tasks.executeTask(task);
		started.execution = execution;
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
};

const selectWorkspaceTask = (
	availableTasks: Task[],
	workspaceFolder: TaskFolder | undefined,
): Task | undefined => {
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
};

export type { TaskRunResult };
export { runTask };
