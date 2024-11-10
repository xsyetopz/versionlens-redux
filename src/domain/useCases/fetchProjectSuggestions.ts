import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import { PackageDependency, PackageResponse, TPackageClientRequest } from '#domain/packages';
import { ISuggestionProvider } from '#domain/providers';
import { FetchPackageSuggestions } from '#domain/useCases';

export class FetchProjectSuggestions {

  constructor(
    private readonly fetchPackageSuggestions: FetchPackageSuggestions,
    private readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fetchPackageSuggestions", fetchPackageSuggestions);
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

    this.logger.debug("queueing %s package fetch tasks", parsedPackages.length);

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
      const promisedFetch = this.fetchPackageSuggestions.execute(provider, clientRequest);

      // queue the fetch task
      promises.push(promisedFetch);
    }

    // parallel the fetch requests
    const responses: Array<PackageResponse> = await Promise.all(promises);

    // report completed duration
    const completedAt = performance.now();
    this.logger.info(
      "all packages fetched for %s (%s ms)",
      provider.name,
      Math.floor(completedAt - startedAt)
    );

    // flatten results
    return responses.flat();
  }

}