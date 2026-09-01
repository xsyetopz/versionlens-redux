import {
  arch as hostArchitecture,
  platform as hostPlatform,
} from "node:process";

const nativeEditorTargets = new Map([
  ["linux-x64", "x86_64-unknown-linux-gnu"],
  ["linux-arm64", "aarch64-unknown-linux-gnu"],
  ["darwin-x64", "x86_64-apple-darwin"],
  ["darwin-arm64", "aarch64-apple-darwin"],
  ["win32-x64", "x86_64-pc-windows-msvc"],
  ["win32-arm64", "aarch64-pc-windows-msvc"],
]);

function resolveNativeEditorTarget(args, packageName) {
  const [requestedPlatform, requestedArchitecture, requestedRustTarget] = args;
  const requestedTarget = args.length > 0;
  if (requestedTarget && args.length !== 3) {
    throw new Error(
      `${packageName} target packaging requires platform, architecture, and Rust target arguments.`,
    );
  }
  if (
    requestedTarget &&
    (requestedPlatform !== hostPlatform ||
      requestedArchitecture !== hostArchitecture)
  ) {
    throw new Error(
      `Cannot package ${requestedPlatform}-${requestedArchitecture} on ${hostPlatform}-${hostArchitecture}; use a native runner.`,
    );
  }

  const platform = requestedPlatform ?? hostPlatform;
  const architecture = requestedArchitecture ?? hostArchitecture;
  const editorTarget = `${platform}-${architecture}`;
  const expectedRustTarget = nativeEditorTargets.get(editorTarget);
  if (!expectedRustTarget) {
    throw new Error(
      `Unsupported ${packageName} package target: ${editorTarget}`,
    );
  }
  if (requestedRustTarget && requestedRustTarget !== expectedRustTarget) {
    throw new Error(
      `${editorTarget} requires Rust target ${expectedRustTarget}, received ${requestedRustTarget}.`,
    );
  }

  const executableName =
    platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
  const sourceParts = ["target"];
  if (requestedRustTarget) {
    sourceParts.push(requestedRustTarget);
  }
  sourceParts.push("release", executableName);

  return {
    architecture,
    editorTarget,
    executableName,
    platform,
    rustTarget: requestedRustTarget,
    sourceParts,
  };
}

export { nativeEditorTargets, resolveNativeEditorTarget };
