import * as vscode from "vscode";

const activeLabels = new Set<string>();

export type TaskRunResult =
	| { exitCode: number | undefined; kind: "completed" }
	| { kind: "busy" }
	| { kind: "notFound" };

export async function runTask(label: string): Promise<TaskRunResult> {
	if (activeLabels.has(label)) {
		return { kind: "busy" };
	}

	const task = (await vscode.tasks.fetchTasks()).find(
		(item) => item.name === label,
	);
	if (!task) {
		vscode.window.showWarningMessage(`Version Lens task not found: ${label}`);
		return { kind: "notFound" };
	}

	activeLabels.add(label);
	let disposable: vscode.Disposable | undefined;
	try {
		const completed = new Promise<TaskRunResult>((resolve) => {
			disposable = vscode.tasks.onDidEndTaskProcess((event) => {
				if (event.execution.task.name !== task.name) {
					return;
				}

				disposable?.dispose();
				activeLabels.delete(label);
				resolve({ exitCode: event.exitCode, kind: "completed" });
			});
		});

		await vscode.tasks.executeTask(task);
		return await completed;
	} catch (error) {
		disposable?.dispose();
		activeLabels.delete(label);
		throw error;
	}
}
