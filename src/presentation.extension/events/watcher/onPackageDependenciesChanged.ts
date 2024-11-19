import type { ILogger } from '#domain/logging';
import { PackageDependency } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnPackageDependenciesChanged {

  constructor(
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("logger", logger);
  }

  async execute(
    provider: ISuggestionProvider,
    packageFilePath: string,
    newDependencies: PackageDependency[]
  ): Promise<void> {


  }

}
