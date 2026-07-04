import { createRequire } from "node:module";
import { join } from "node:path";
import type { NativeSessionConfig } from "./config.ts";
import type { NativeApplyCommandInput, NativeDocumentInput } from "./input.ts";
import type { AnalyzeDocumentOutput, ResolveDocumentOutput } from "./output.ts";

export type NativeSession = {
	analyzeDocument(input: NativeDocumentInput): AnalyzeDocumentOutput;
	applyCommand(input: NativeApplyCommandInput): ResolveDocumentOutput;
	clearCache(): void;
	disposeSession(): void;
	resolveDocument(input: NativeDocumentInput): Promise<ResolveDocumentOutput>;
};

export type NativeModule = {
	createSession(config: NativeSessionConfig): NativeSession;
};

export function loadNative(extensionPath: string): NativeModule {
	return createRequire(join(extensionPath, "dist", "extension.js"))(
		join(extensionPath, "native", "versionlens_napi.node"),
	) as NativeModule;
}
