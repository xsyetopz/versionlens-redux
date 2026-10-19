#!/usr/bin/env bun
import { constants as bufferConstants } from "node:buffer";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { basename } from "node:path";

const [, , vsixPath, serverPath, target] = Bun.argv;
if (!(vsixPath && serverPath && target)) {
  throw new Error(
    "usage: bun scripts/package/vsix.mjs <vsix> <server-binary> <vscode-target>",
  );
}

function digest(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

const serverEntry = `extension/server/${basename(serverPath)}`;
const packagedServer = execFileSync("unzip", ["-p", vsixPath, serverEntry], {
  maxBuffer: bufferConstants.MAX_LENGTH,
});
if (digest(packagedServer) !== digest(readFileSync(serverPath))) {
  throw new Error(`${vsixPath} ${serverEntry} does not match ${serverPath}`);
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
