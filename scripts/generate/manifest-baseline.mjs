// Compares the `versionlens.*` settings of the 0.4.4 release with the
// committed ones: no key is removed, no type changes, and a default only
// gains entries, and only for the settings that new manifest support grows.
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { ROOT } from "./manifests.mjs";

const BASELINE_TAG = "v0.4.4";
const PACKAGE_JSON = "packages/vscode-extension/package.json";

/** The exact entries that each grown default gains over 0.4.4. */
const GROWN_DEFAULTS = new Map([
  ["versionlens.dotnet.files", ["**/dotnet-tools.json"]],
  ["versionlens.cargo.files", ["**/.cargo/config.toml", "**/.cargo/config"]],
  ["versionlens.swift.files", ["**/Cartfile", "**/Cartfile.private"]],
  ["versionlens.cargo.dependencyProperties", ["patch.*", "patch.*.*"]],
]);

function settingsOf(text) {
  const properties = JSON.parse(text).contributes.configuration.properties;
  return new Map(
    Object.entries(properties).filter(([key]) =>
      key.startsWith("versionlens."),
    ),
  );
}

/** The glob alternatives of a `files` default, with braces expanded. */
function expandBraces(glob) {
  const match = /\{([^{}]*)\}/u.exec(glob);
  if (!match) {
    return [glob];
  }
  return match[1]
    .split(",")
    .flatMap((option) =>
      expandBraces(
        glob.slice(0, match.index) +
          option +
          glob.slice(match.index + match[0].length),
      ),
    );
}

/** The entries of a default, or null when it is not a list or a glob. */
function entriesOf(value) {
  if (Array.isArray(value)) {
    return value.map(String);
  }
  return typeof value === "string" ? expandBraces(value) : null;
}

export function compareSettings(baseline, current) {
  const problems = [];
  const grown = [];
  let preserved = 0;
  for (const [key, old] of baseline) {
    const next = current.get(key);
    if (!next) {
      problems.push(`removed key ${key}`);
      continue;
    }
    if (JSON.stringify(old.type) !== JSON.stringify(next.type)) {
      problems.push(
        `changed type of ${key}: ${JSON.stringify(old.type)} -> ${JSON.stringify(next.type)}`,
      );
      continue;
    }
    preserved += 1;
    if (JSON.stringify(old.default) === JSON.stringify(next.default)) {
      continue;
    }
    const before = entriesOf(old.default);
    const after = entriesOf(next.default);
    const kept = before && after && before.every((e) => after.includes(e));
    if (!kept) {
      problems.push(`changed default of ${key}`);
      continue;
    }
    const added = after.filter((entry) => !before.includes(entry));
    if (added.length === 0) {
      continue;
    }
    const expected = GROWN_DEFAULTS.get(key);
    if (expected && [...expected].sort().join() === [...added].sort().join()) {
      grown.push({ key, added });
    } else {
      problems.push(
        `unexpected new default entries for ${key}: ${added.join(", ")}`,
      );
    }
  }
  for (const key of GROWN_DEFAULTS.keys()) {
    if (!grown.some((item) => item.key === key)) {
      problems.push(`expected ${key} to grow, but it did not`);
    }
  }
  return { problems, grown, preserved };
}

if (import.meta.main) {
  const shown = spawnSync("git", ["show", `${BASELINE_TAG}:${PACKAGE_JSON}`], {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (shown.status !== 0) {
    console.error(
      `cannot read ${PACKAGE_JSON} at ${BASELINE_TAG}: ${shown.stderr}`,
    );
    process.exit(2);
  }
  const baseline = settingsOf(shown.stdout);
  const current = settingsOf(readFileSync(join(ROOT, PACKAGE_JSON), "utf8"));
  const { problems, grown, preserved } = compareSettings(baseline, current);
  const added = [...current.keys()].filter((key) => !baseline.has(key));
  console.log(
    `${BASELINE_TAG}: ${baseline.size} keys, ${preserved} preserved, ${added.length} added`,
  );
  for (const { key, added: entries } of grown) {
    console.log(`grown ${key}: +${entries.join(", ")}`);
  }
  for (const problem of problems) {
    console.error(problem);
  }
  if (problems.length > 0) {
    process.exit(1);
  }
}
