import { basename, dirname } from "node:path";
import { RelativePattern, type Uri, window, workspace } from "#vscode-host";
import { enabledFilePatternKeys } from "../config/keys/files.ts";
import { fileDocument } from "../documents/file.ts";

const defaultExcludes = [
  "**/node_modules/**",
  "**/bower_components/**",
  "**/bin/**",
  "**/.git/**",
  "**/.vscode/**",
] as const;

interface PackageFilePattern {
  exclude: RelativePattern | undefined;
  pattern: RelativePattern;
}

function packageFilePatterns(): PackageFilePattern[] {
  if ((workspace.workspaceFolders?.length ?? 0) === 0) {
    const document = fileDocument(window.activeTextEditor?.document);
    if (!document) {
      return [];
    }
    return [{ exclude: undefined, pattern: packageFilePattern(document.uri) }];
  }

  return (workspace.workspaceFolders ?? []).flatMap(
    (folder): PackageFilePattern[] => {
      const resource = folder.uri;
      const config = workspace.getConfiguration("versionlens", resource);
      const fileExcludes = workspace
        .getConfiguration("files", resource)
        .get<Record<string, boolean>>("exclude", {});
      const editorExcludes = Object.entries(fileExcludes ?? {})
        .filter(([, enabled]): boolean => enabled)
        .map(([pattern]): string => pattern);

      return enabledFilePatternKeys(
        config.get<string[]>("enabledProviders"),
      ).map(([, key, , excludePatterns]): PackageFilePattern => {
        const pattern = config.get<string>(key, "**/*") ?? "**/*";
        const excludes = [
          ...defaultExcludes,
          ...editorExcludes,
          ...(excludePatterns ?? []),
        ];
        return {
          exclude: new RelativePattern(folder, mapToSinglePattern(excludes)),
          pattern: new RelativePattern(folder, pattern),
        };
      });
    },
  );
}

function packageFilePattern(uri: Uri): RelativePattern {
  return new RelativePattern(dirname(uri.fsPath), basename(uri.fsPath));
}

function mapToSinglePattern(patterns: readonly string[]): string {
  if (patterns.length === 1) {
    return patterns[0] ?? "";
  }
  return `{${patterns.join(",")}}`;
}

function isDefaultExcluded(uri: Uri): boolean {
  const path = uri.fsPath.replaceAll("\\", "/");
  return ["node_modules", "bower_components", "bin", ".git", ".vscode"].some(
    (segment): boolean => path.includes(`/${segment}/`),
  );
}

export { isDefaultExcluded, packageFilePattern, packageFilePatterns };
