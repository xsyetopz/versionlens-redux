import type { NativeApplyCommand, ResolveDocumentOutput } from "../native.ts";

type AuthenticationResolution = Promise<ResolveDocumentOutput | undefined>;
type CodeLensReplacementMode = "disable" | "disableThenEnable" | "preserve";

interface ApplyOptions {
  ignoreCodeLensReplace?: boolean;
  selectedVersion?: string | undefined;
}

interface ApplySelection {
  command: NativeApplyCommand | undefined;
  dependencyName: string | undefined;
  selectedVersion: string | undefined;
}

export type {
  ApplyOptions,
  ApplySelection,
  AuthenticationResolution,
  CodeLensReplacementMode,
};
