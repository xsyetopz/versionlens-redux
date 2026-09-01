#!/usr/bin/env bun
import { readFileSync, writeFileSync } from "node:fs";

const [, , registryPath, version] = Bun.argv;
if (!registryPath || !version) {
  throw new Error(
    "usage: bun scripts/release/zed-registry.mjs <extensions.toml> <version>",
  );
}

const extensionId = "versionlens-lsp";
const entry = `[${extensionId}]\nsubmodule = "extensions/${extensionId}"\npath = "packages/zed-extension"\nversion = "${version}"\n`;
const source = readFileSync(registryPath, "utf8");
const sectionPattern = new RegExp(
  `^\\[${extensionId}\\]\\n(?:^(?!\\[).*(?:\\n|$))*`,
  "mu",
);
const next = sectionPattern.test(source)
  ? source.replace(sectionPattern, entry)
  : `${source.trimEnd()}\n\n${entry}`;
writeFileSync(registryPath, next);
