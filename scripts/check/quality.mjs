#!/usr/bin/env bun
import process from "node:process";
import { runRepositoryCheck } from "./quality/repository.mjs";

const result = runRepositoryCheck();
const message = result.error ?? result.output;
if (message) {
  console.error(message);
}
if (result.exitCode !== 0) {
  process.exit(result.exitCode);
}
