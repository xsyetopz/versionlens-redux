import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type TPackageClientRequest,
  type TPackageClientResponse,
  type TPackageClientResponseStatus,
  ClientResponseFactory,
  PackageSourceType,
  PackageVersionType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import {
  type INpmRegistry,
  type NpaSpec,
  type TNpmRegistryClientResponse,
  NpaTypes,
  NpmConfig
} from '#domain/providers/npm';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import semver from 'semver';

export class NpmRegistryClient {

  constructor(
    readonly npmRegistryFetch: INpmRegistry,
    readonly config: NpmConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("npmRegistryFetch", npmRegistryFetch);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(
    request: TPackageClientRequest<any>,
    npaSpec: NpaSpec
  ): Promise<TPackageClientResponse> {
    const type: PackageVersionType = <any>npaSpec.type;

    const spec = type == PackageVersionType.Alias
      ? npaSpec.subSpec as NpaSpec
      : npaSpec;

    const requestedPackage = request.parsedDependency.package;

    // fetch the package from the npm's registry
    const response = await this.request(spec, request.clientData);

    const { compareLoose } = semver;

    let versionRange = spec.rawSpec;

    const resolved = {
      name: spec.name,
      version: versionRange,
    };

    const responseStatus: TPackageClientResponseStatus = {
      source: response.source,
      status: response.status,
    };

    const packumentResponse = response.data;

    // extract raw versions and sort
    const rawVersions = Object.keys(packumentResponse.versions || {}).sort(compareLoose);

    // seperate versions to releases and prereleases
    let { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      rawVersions,
      this.config.prereleaseTagFilter
    );

    // extract prereleases from dist tags
    const distTags = packumentResponse['dist-tags'] || {};
    const latestTaggedVersion = distTags['latest'];

    // extract releases
    if (latestTaggedVersion) {
      // cap the releases to the latest tagged version
      releases = VersionUtils.lteFromArray(
        releases,
        latestTaggedVersion
      );
    }

    // use 'latest' tagged version from author?
    const suggestLatestVersion = latestTaggedVersion || (
      releases.length > 0 ?
        // suggest latest release?
        releases[releases.length - 1] :
        // no suggestion
        null
    );

    if (npaSpec.type === NpaTypes.Tag) {

      // get the tagged version. eg latest|next
      versionRange = distTags[requestedPackage.version];
      if (!versionRange) {

        // No match
        return ClientResponseFactory.createNoMatch(
          PackageSourceType.Registry,
          type,
          responseStatus,
          suggestLatestVersion
        );

      }

    }

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases,
      suggestLatestVersion
    );

    return {
      source: PackageSourceType.Registry,
      responseStatus,
      type,
      resolved,
      suggestions,
    };

  }

  async request(npaSpec: NpaSpec, clientData: any): Promise<TNpmRegistryClientResponse> {
    try {
      const registry = this.npmRegistryFetch.pickRegistry(npaSpec, clientData);
      const url = `${ensureEndSlash(registry)}${npaSpec.escapedName}`;
      const registryResponse = await this.npmRegistryFetch.json(url, clientData);
      const result: TNpmRegistryClientResponse = {
        source: ClientResponseSource.remote,
        status: 200,
        data: registryResponse,
        rejected: false
      };

      return result;
    } catch (error) {
      const result: TNpmRegistryClientResponse = {
        source: ClientResponseSource.remote,
        status: error.code,
        data: error.message,
        rejected: true
      };

      throw result;
    }

  }

}