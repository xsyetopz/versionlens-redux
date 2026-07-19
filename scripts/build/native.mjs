#!/usr/bin/env bun

import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

let profile = "debug";
if (Bun.argv.includes("--release")) {
  profile = "release";
}
const targetIndex = Bun.argv.indexOf("--target");
let rustTarget;
if (targetIndex !== -1) {
  rustTarget = Bun.argv[targetIndex + 1];
}
if (targetIndex !== -1 && !rustTarget) {
  throw new Error("--target requires a Rust target triple");
}
const targetPlatform = rustTarget ?? process.platform;
let ext = "so";
if (targetPlatform.includes("windows")) {
  ext = "dll";
} else if (targetPlatform.includes("darwin")) {
  ext = "dylib";
}
let libraryName = `libversionlens_napi.${ext}`;
if (rustTarget?.includes("windows")) {
  libraryName = "versionlens_napi.dll";
}
const sourceParts = [Bun.env.CARGO_TARGET_DIR ?? "target"];
if (rustTarget) {
  sourceParts.push(rustTarget);
}
sourceParts.push(profile, libraryName);
const source = join(...sourceParts);
const target = join(
  "packages",
  "vscode-extension",
  "native",
  "versionlens_napi.node",
);

mkdirSync(join("packages", "vscode-extension", "native"), { recursive: true });
copyFileSync(source, target);
