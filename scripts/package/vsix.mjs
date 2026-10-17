#!/usr/bin/env bun
import { constants as bufferConstants } from "node:buffer";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const [, , vsixPath, nativePath, target] = Bun.argv;
if (!(vsixPath && nativePath && target)) {
  throw new Error(
    "usage: bun scripts/package/vsix.mjs <vsix> <native-library> <vscode-target>",
  );
}

function digest(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

const packagedNative = execFileSync(
  "unzip",
  ["-p", vsixPath, "extension/native/versionlens_napi.node"],
  { maxBuffer: bufferConstants.MAX_LENGTH },
);
if (digest(packagedNative) !== digest(readFileSync(nativePath))) {
  throw new Error(
    `${vsixPath} native/versionlens_napi.node does not match ${nativePath}`,
  );
}
const manifest = execFileSync(
  "unzip",
  ["-p", vsixPath, "extension.vsixmanifest"],
  {
    encoding: "utf8",
  },
);
if (!manifest.includes(`TargetPlatform="${target}"`)) {
  throw new Error(`${vsixPath} target is not ${target}`);
}
