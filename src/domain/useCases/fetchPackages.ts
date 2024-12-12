import type { ILogger } from '#domain/logging';
import { type PackageResponse, type TPackageClientRequest, PackageDependency } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { FetchPackage } from '#domain/useCases';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class FetchPackages {

  constructor(
    private readonly fetchPackage: FetchPackage,
    private readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fetchPackage", fetchPackage);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(
    provider: ISuggestionProvider,
    projectPath: string,
    packagePath: string,
    parsedPackages: Array<PackageDependency>,
  ): Promise<Array<PackageResponse>> {

    // get any client data if implemented
    let clientData: any = {};
    if (provider.preFetchSuggestions) {
      clientData = await provider.preFetchSuggestions(projectPath, packagePath);
    }

    this.logger.debug(
      "queueing {packageCount} package fetch tasks",
      parsedPackages.length
    );

    // capture start time
    const startedAt = performance.now();

    // queue package fetch tasks
    const promises = [];
    for (const parsedPackage of parsedPackages) {
      // setup the client request
      const clientRequest: TPackageClientRequest<any> = {
        providerName: provider.name,
        clientData,
        parsedDependency: parsedPackage,
        attempt: 0
      };

      // get the fetch task
      const promisedFetch = this.fetchPackage.execute(provider, clientRequest);

      // queue the fetch task
      promises.push(promisedFetch);
    }

    // parallel the fetch requests
    const responses: Array<PackageResponse> = await Promise.all(promises);

    // report completed duration
    const completedAt = performance.now();
    this.logger.info(
      "all packages fetched for {providerName} ({duration} ms)",
      provider.name,
      Math.floor(completedAt - startedAt)
    );

    // flatten results
    return responses.flat();
  }

}