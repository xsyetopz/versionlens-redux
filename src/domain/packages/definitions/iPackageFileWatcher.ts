import { PackageDependency } from '#domain/packages';
import { ISuggestionProvider } from "domain/providers";
import { Uri } from "vscode";

export type OnPackageDependenciesChangedEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
  packageDeps: PackageDependency[]
) => Promise<void>;

export interface IPackageFileWatcher {

  watchFolder(): Promise<void>;

  watchFile(file: Uri): Promise<void>

  watch: () => void;

  registerListener: (
    listener: OnPackageDependenciesChangedEvent,
    thisArg: any
  ) => void;

}