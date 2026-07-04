import { expect, mock, test } from "bun:test";

let fileType = 1;
const openedExternal: unknown[] = [];
const openedTextDocuments: unknown[] = [];

mock.module("vscode", () => ({
	FileType: {
		Directory: 2,
		File: 1,
	},
	Uri: {
		file: (path: string) => ({ path, scheme: "file" }),
	},
	env: {
		openExternal(uri: unknown) {
			openedExternal.push(uri);
		},
	},
	window: {
		showTextDocument(uri: unknown) {
			openedTextDocuments.push(uri);
		},
	},
	workspace: {
		fs: {
			stat: () => ({ type: fileType }),
		},
	},
}));

test("open dependency opens file paths in the editor", async () => {
	const { openDependency } = await import("./open.ts");
	reset();
	fileType = 1;

	await openDependency("/repo/local/package.json");

	expect(openedTextDocuments).toEqual([
		{ path: "/repo/local/package.json", scheme: "file" },
	]);
	expect(openedExternal).toEqual([]);
});

test("open dependency opens directory paths externally", async () => {
	const { openDependency } = await import("./open.ts");
	reset();
	fileType = 2;

	await openDependency("/repo/local");

	expect(openedExternal).toEqual([{ path: "/repo/local", scheme: "file" }]);
	expect(openedTextDocuments).toEqual([]);
});

function reset() {
	openedExternal.length = 0;
	openedTextDocuments.length = 0;
}
