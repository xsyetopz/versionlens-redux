import { HttpRequestError } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageResponse,
  ClientResponseFactory,
  PackageCache,
  PackageSourceType,
  PackageStatusFactory,
  ResponseFactory,
  getProjectVersionSuggestions
} from '#domain/packages';
import { PackageDescriptorType } from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Use case for fetching suggestions for a single package.
 */
export class FetchPackage {

  /**
   * Initializes a new instance of the FetchPackage class.
   * @param packageCache The cache for package suggestion responses.
   * @param logger The logger to use.
   */
  constructor(
    private readonly packageCache: PackageCache,
    private readonly logger: ILogger
  ) {
    throwUndefinedOrNull("packageCache", packageCache);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes the fetch package use case.
   * @param provider The suggestion provider to use.
   * @param request The package client request.
   * @returns A promise resolving to an array of package responses.
   */
  async execute(
    provider: ISuggestionProvider,
    request: PackageClientRequest<any>
  ): Promise<Array<PackageResponse>> {
    const providerName = provider.name;
    const parsedDependency = request.parsedDependency;
    const requestedPackage = parsedDependency.package;

    // capture start time
    const startedAt = performance.now();

    // check if this is a project version dep
    if (parsedDependency.descriptors.hasType(PackageDescriptorType.projectVersion)) {
      return getProjectVersionSuggestions(parsedDependency.package.version)
        .map(
          suggestion => ResponseFactory.createProjectVersionPackageResponse(
            providerName, request, suggestion
          )
        );
    }

    // get the package from the client or the cache
    let source = "cache";
    const response = await this.packageCache.getOrCreate(
      providerName,
      requestedPackage,
      async () => {
        source = "client";
        return await this.fetch(provider, request);
      },
      provider.config.caching.duration
    );

    // report completed duration
    const completedAt = performance.now();
    this.logger.info(
      'fetched from {source} {packageName}@{packageVersion} ({duration} ms)',
      source,
      requestedPackage.name,
      requestedPackage.version,
      Math.floor(completedAt - startedAt)
    );

    return ResponseFactory.createSuccess(
      providerName,
      request,
      response
    );
  }

  /**
   * Performs the actual fetch operation using the provider.
   * @param provider The suggestion provider.
   * @param request The package client request.
   * @returns A promise resolving to the package client response.
   */
  async fetch(
    provider: ISuggestionProvider,
    request: PackageClientRequest<any>
  ): Promise<PackageClientResponse> {

    const requestedPackage = request.parsedDependency.package;

    this.logger.trace("fetching {packageName}", requestedPackage.name);

    try {
      // fetch the package
      return await provider.fetchSuggestions(request);
    } catch (error) {
      if (error instanceof HttpRequestError) {
        const suggestion = PackageStatusFactory.createFromHttpStatus(error.status);
        if (suggestion != null) {
          this.logger.error(
            "{packageName}@{packageVersion} was rejected with the status code {responseStatus}",
            requestedPackage.name,
            requestedPackage.version,
            error.status
          );
          return ClientResponseFactory.create(
            PackageSourceType.Registry,
            error,
            [suggestion]
          )
        }
      }

      // unexpected error
      this.logger.error(
        `{functionName} caught an exception.\n Package: {requestedPackage}\n Error: {error}`,
        this.fetch.name,
        requestedPackage,
        error
      );
      throw error;
    }

  }

}