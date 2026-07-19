#!/usr/bin/env bun

import process from "node:process";
import { runRepositoryCheck } from "./quality/repository.mjs";

const result = runRepositoryCheck();
const message = result.error ?? result.output;
if (message) {
  console.error(message);
}
process.exit(result.exitCode);
