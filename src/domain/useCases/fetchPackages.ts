import type { ILogger } from '#domain/logging';
import { type PackageClientRequest, type PackageResponse, PackageDependency } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { FetchPackage } from '#domain/useCases';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Use case for fetching suggestions for multiple packages in parallel.
 */
export class FetchPackages {

  /**
   * Initializes a new instance of the FetchPackages class.
   * @param fetchPackage The use case for fetching a single package.
   * @param logger The logger to use.
   */
  constructor(
    private readonly fetchPackage: FetchPackage,
    private readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fetchPackage", fetchPackage);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes the fetch packages use case.
   * @param provider The suggestion provider to use.
   * @param projectPath The path to the project.
   * @param packagePath The path to the package file.
   * @param parsedPackages The list of parsed dependencies to fetch.
   * @returns A promise resolving to a flattened array of package responses.
   */
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
      const clientRequest: PackageClientRequest<any> = {
        providerName: provider.name,
        clientData,
        parsedDependency: parsedPackage
      };

      // get the fetch task
      const promisedFetch = this.fetchPackage.execute(provider, clientRequest);

      // queue the fetch task
      promises.push(promisedFetch);
    }

    // parallel the fetch requests
    const responses = await Promise.all(promises);

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