import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  ClientResponseFactory,
  createSuggestions,
  PackageSourceType,
  PackageVersionType,
  VersionUtils
} from '#domain/packages';
import type { DenoConfig, JsrClient } from '#domain/providers/deno';
import type { NpaSpec, NpmClientData, NpmSuggestionResolver } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Resolves package suggestions for Deno dependencies, supporting both NPM and JSR registries.
 */
export class DenoSuggestionResolver {

  /**
   * Initializes a new instance of the DenoSuggestionResolver class.
   * @param config The configuration for the Deno provider.
   * @param jsrClient The client used to interact with JSR.
   * @param npmSuggestionResolver The NPM suggestion resolver.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: DenoConfig,
    readonly jsrClient: JsrClient,
    readonly npmSuggestionResolver: NpmSuggestionResolver,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsrClient', jsrClient);
    throwUndefinedOrNull('npmSuggestionResolver', npmSuggestionResolver);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Resolves suggestions from the NPM registry.
   * @param request The package client request.
   * @param npaSpec The NPM package specification.
   * @returns A promise resolving to the package client response.
   */
  async fromNpm(request: PackageClientRequest<NpmClientData>, npaSpec: NpaSpec) {
    return this.npmSuggestionResolver.fromRegistry(request, npaSpec);
  }

  /**
   * Resolves suggestions from the JSR registry.
   * @param npaSpec The NPM package specification (used for JSR packages by mapping).
   * @returns A promise resolving to the package client response.
   */
  async fromJsr(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    // fetch
    const jsonResponse = await this.jsrClient.get(npaSpec.subSpec.name);

    // process response
    const versionRange = npaSpec.subSpec.rawSpec;
    const resolved = {
      name: npaSpec.subSpec.name,
      version: versionRange,
    };

    // sort versions
    const rawVersions = jsonResponse.data.toSorted(VersionUtils.compareVersionsAndBuilds);

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(rawVersions);

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      semverVersions,
      this.config.prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    return {
      source: PackageSourceType.Registry,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type: PackageVersionType.Alias,
      resolved,
      suggestions,
    };
  }

}