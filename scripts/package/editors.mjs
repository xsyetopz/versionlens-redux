#!/usr/bin/env bun
import { existsSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import process from "node:process";

const REQUIRED_JAVA_MAJOR = 21;
const JAVA_VERSION_PATTERN = /version\s+"(?<major>\d+)/u;

function run(command, options = {}) {
  console.log(`$ ${command.join(" ")}`);
  const result = Bun.spawnSync(command, {
    stderr: "inherit",
    stdout: "inherit",
    ...options,
  });
  if (result.exitCode !== 0) {
    throw new Error(
      `Command failed with exit code ${result.exitCode ?? 1}: ${command.join(" ")}`,
    );
  }
}

function commandOutput(command) {
  const result = Bun.spawnSync(command, { stderr: "pipe", stdout: "pipe" });
  if (result.exitCode !== 0) {
    return "";
  }
  return new TextDecoder().decode(result.stdout).trim();
}

function javaMajor(candidateHome) {
  const result = Bun.spawnSync(
    [join(candidateHome, "bin", "java"), "-version"],
    {
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  if (result.exitCode !== 0) {
    return null;
  }
  const output = `${new TextDecoder().decode(result.stderr)}${new TextDecoder().decode(result.stdout)}`;
  const match = output.match(JAVA_VERSION_PATTERN);
  if (match === null || match.groups === undefined) {
    return null;
  }
  return Number(match.groups.major);
}

function candidateJavaHomes() {
  const candidates = [];
  if (Bun.env.JAVA_HOME) {
    candidates.push([Bun.env.JAVA_HOME, "JAVA_HOME"]);
  }
  if (process.platform === "darwin" && existsSync("/usr/libexec/java_home")) {
    const javaHome = commandOutput(["/usr/libexec/java_home", "-v", "21"]);
    if (javaHome) {
      candidates.push([javaHome, "macOS java_home"]);
    }
  }
  for (const prefix of ["/opt/homebrew", "/usr/local"]) {
    candidates.push([
      join(prefix, "opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"),
      `${prefix}/opt/openjdk@21`,
    ]);
  }
  const sdkmanJavaHome = join(homedir(), ".sdkman/candidates/java");
  if (existsSync(sdkmanJavaHome)) {
    for (const name of readdirSync(sdkmanJavaHome)) {
      if (name.startsWith("21")) {
        candidates.push([
          join(sdkmanJavaHome, name, "current"),
          "SDKMAN Java 21",
        ]);
      }
    }
  }
  const pathJava = commandOutput(["sh", "-c", "command -v java"]);
  if (pathJava) {
    candidates.push([dirname(dirname(pathJava)), "PATH java"]);
  }
  return candidates;
}

function resolveJava21() {
  const rejected = [];
  for (const [candidate, label] of candidateJavaHomes()) {
    if (existsSync(join(candidate, "bin", "java"))) {
      const major = javaMajor(candidate);
      if (major === REQUIRED_JAVA_MAJOR) {
        return candidate;
      }
      let version = "unreadable";
      if (major !== null) {
        version = `Java ${major}`;
      }
      rejected.push(`${label}: ${version}`);
    }
  }
  let diagnostic = "No Java runtime candidates were found.";
  if (rejected.length > 0) {
    diagnostic = `Rejected candidates: ${rejected.join(", ")}`;
  }
  throw new Error(
    [
      "Editor packaging requires an installed Java 21 runtime.",
      "No unpinned JDK download is attempted.",
      "Set JAVA_HOME to a Java 21 installation or install the repository-managed toolchain used by CI (actions/setup-java Java 21).",
      diagnostic,
    ].join(" "),
  );
}

const javaRuntime = resolveJava21();
console.log(`Using Java ${REQUIRED_JAVA_MAJOR} runtime: ${javaRuntime}`);
const repositoryRoot = process.cwd();
const fixturesRoot = join(repositoryRoot, "tests", "fixtures");
const projectGradleState = join(
  repositoryRoot,
  "packages",
  "jetbrains-plugin",
  ".gradle",
);
const projectBuildTemp = join(
  repositoryRoot,
  "packages",
  "jetbrains-plugin",
  "build",
  "tmp",
);
const gradleHome = mkdtempSync(join(tmpdir(), "versionlens-gradle-"));
if (gradleHome.startsWith(fixturesRoot)) {
  throw new Error(`Refusing Gradle user home inside fixtures: ${gradleHome}`);
}
const gradleEnvironment = {
  ...Bun.env,
};
gradleEnvironment.JAVA_HOME = javaRuntime;
gradleEnvironment.PATH = `${join(javaRuntime, "bin")}${delimiter}${Bun.env.PATH ?? ""}`;
gradleEnvironment.GRADLE_USER_HOME = gradleHome;
const vsixFreshnessCommand = ["bun", "scripts/check/vsix.mjs"];
const editorFreshnessCommand = ["bun", "scripts/check/editors.mjs"];

try {
  run(["bun", "run", "package"], { env: gradleEnvironment });
  run(vsixFreshnessCommand, { env: gradleEnvironment });
  run(["cargo", "build", "-p", "versionlens-lsp", "--release", "--locked"], {
    env: gradleEnvironment,
  });
  run(
    [
      "cargo",
      "build",
      "--manifest-path",
      "packages/zed-extension/Cargo.toml",
      "--release",
      "--locked",
    ],
    { env: gradleEnvironment },
  );
  run(["bun", "scripts/package/zed.mjs"], { env: gradleEnvironment });
  run(["bun", "scripts/package/neovim.mjs"], { env: gradleEnvironment });
  run(["bun", "scripts/check/neovim.mjs"], { env: gradleEnvironment });
  run(
    [
      "./packages/jetbrains-plugin/gradlew",
      "-p",
      "packages/jetbrains-plugin",
      "buildPlugin",
      "--no-daemon",
    ],
    { env: gradleEnvironment },
  );
  run(editorFreshnessCommand, { env: gradleEnvironment });
} finally {
  rmSync(gradleHome, { force: true, recursive: true });
  rmSync(projectGradleState, { force: true, recursive: true });
  rmSync(projectBuildTemp, { force: true, recursive: true });
}
