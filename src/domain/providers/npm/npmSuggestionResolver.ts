import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageResource,
  ClientResponseFactory,
  createSuggestions,
  PackageClientResponseStatus,
  PackageSourceType,
  PackageVersionType,
  VersionUtils
} from '#domain/packages';
import {
  type NpaSpec,
  type NpmClientData,
  type NpmConfig,
  type NpmGitHubClient,
  type NpmRegistryClient,
  NpaTypes
} from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { prerelease } from 'semver';

export class NpmSuggestionResolver {

  constructor(
    readonly config: NpmConfig,
    readonly npmRegistryClient: NpmRegistryClient,
    readonly npmGithubClient: NpmGitHubClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('npmRegistryClient', npmRegistryClient);
    throwUndefinedOrNull('npmGithubClient', npmGithubClient);
    throwUndefinedOrNull('logger', logger);
  }

  async fromFileProtocol(pkg: PackageResource): Promise<PackageClientResponse> {
    return await ClientResponseFactory.createDirectoryFromFileProtocol(pkg);
  }

  async fromGit(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    if (!npaSpec.hosted) {
      // could not resolve
      throw {
        status: 'EUNSUPPORTEDPROTOCOL',
        data: 'Git url could not be resolved',
        source: ClientResponseSource.local
      };
    }

    if (!npaSpec.gitCommittish && npaSpec.hosted.default !== 'shortcut') {
      return ClientResponseFactory.createGit();
    }

    // resolve tags, committishes
    return await this.npmGithubClient.fetchGithub(npaSpec);
  }

  async fromRegistry(
    request: PackageClientRequest<NpmClientData>,
    npaSpec: NpaSpec
  ): Promise<PackageClientResponse> {
    const type: PackageVersionType = <any>npaSpec.type;

    const spec = type == PackageVersionType.Alias
      ? npaSpec.subSpec as NpaSpec
      : npaSpec;

    const requestedPackage = request.parsedDependency.package;

    // fetch the package from the npm's registry
    const response = await this.npmRegistryClient.get(spec, request.clientData);

    let versionRange = spec.rawSpec;

    const resolved = {
      name: spec.name,
      version: versionRange,
    };

    const responseStatus: PackageClientResponseStatus = {
      source: response.source,
      status: response.status,
    };

    const packumentResponse = response.data;

    // extract raw versions and sort
    const rawVersions = packumentResponse.versions
      .sort(VersionUtils.compareVersionsAndBuilds);

    // seperate versions to releases and prereleases
    let { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      rawVersions,
      this.config.prereleaseTagFilter
    );

    // extract latest from dist tags
    const distTags = packumentResponse['dist-tags'];
    const latestTaggedVersion = distTags['latest'];
    const latestIsPrerelease = prerelease(latestTaggedVersion) !== null;
    if (latestTaggedVersion && latestIsPrerelease === false) {
      // cap the releases to the latest tagged version
      releases = VersionUtils.lteFromArray(releases, latestTaggedVersion);
    }

    // use 'latest' tagged version from author?
    const suggestLatestVersion = latestTaggedVersion || (
      releases.length > 0
        // suggest latest release?
        ? releases[releases.length - 1]
        // suggest latest prerelease?
        : prereleases.length > 0
          ? prereleases[prereleases.length - 1]
          // no suggestion
          : undefined
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
      versionRange === '*' ? suggestLatestVersion : versionRange,
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

}