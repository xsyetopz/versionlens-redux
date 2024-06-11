import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import { PackageDependency } from '#domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { VersionLensState } from 'presentation.extension';

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
