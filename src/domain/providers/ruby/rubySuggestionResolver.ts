import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageClientResponse,
  type SemverSpec,
  ClientResponseFactory,
  PackageSourceType,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import { RubyConfig, RubyGitHubClient, RubyHttpClient } from '#domain/providers/ruby';
import { parseGitHubRepo } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Resolves package suggestions for Ruby dependencies from the RubyGems registry or GitHub.
 */
export class RubySuggestionResolver {

  /**
   * Initializes a new instance of the RubySuggestionResolver class.
   * @param config The configuration for the Ruby provider.
   * @param rubyHttpClient The client used to interact with RubyGems.
   * @param githubClient The client used to interact with GitHub.
   * @param logger The logger for this resolver.
   */
  constructor(
    readonly config: RubyConfig,
    readonly rubyHttpClient: RubyHttpClient,
    readonly githubClient: RubyGitHubClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("rubyHttpClient", rubyHttpClient);
    throwUndefinedOrNull("githubClient", githubClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Resolves suggestions from a local path.
   * @param packageName The name of the package.
   * @param packageFilePath The path to the Gemfile.
   * @param path The local path to the package.
   * @returns A promise resolving to the package client response.
   */
  async fromPath(
    packageName: string,
    packageFilePath: string,
    path: string
  ): Promise<PackageClientResponse> {
    return await ClientResponseFactory.createDirectory(
      packageName,
      packageFilePath,
      path
    );
  }

  /**
   * Resolves suggestions from a Git repository.
   * @returns A package client response.
   */
  fromGit(): PackageClientResponse {
    return ClientResponseFactory.createGit();
  }

  /**
   * Resolves suggestions from a Git repository.
   * @param gitUrl The Git URL.
   * @param gitRef The Git reference (optional).
   * @param isTag Whether the reference is a tag.
   * @returns A promise resolving to the package client response.
   */
  async fromGitHub(gitUrl: string, gitRef: string, isTag: boolean = false): Promise<PackageClientResponse> {
    const repo = parseGitHubRepo(gitUrl);
    if (!repo) return ClientResponseFactory.createGit();

    if (isTag) {
      // resolve as tag
      return await this.githubClient.fetchTags(
        repo.user,
        repo.project,
        gitRef,
        this.config.prereleaseTagFilter
      );
    }

    if (!gitRef) {
      // resolve commits (suggests latest)
      return await this.githubClient.fetchCommits(
        repo.user,
        repo.project,
        ''
      );
    }

    // fall back to resolving as commit
    return await this.githubClient.fetchCommits(
      repo.user,
      repo.project,
      gitRef
    );
  }

  /**
   * Resolves suggestions from the RubyGems API.
   * @param request The package client request.
   * @param semverSpec The parsed semver specification.
   * @returns A promise resolving to the package client response.
   */
  async resolve(
    request: PackageClientRequest<null>,
    semverSpec: SemverSpec
  ): Promise<PackageClientResponse> {
    const requestPackage = request.parsedDependency.package;
    const httpResponse = await this.rubyHttpClient.get(requestPackage.name);

    // process response
    const versionRange = semverSpec.rawVersion;
    const resolved = {
      name: requestPackage.name,
      version: versionRange,
    };

    // extract semver versions only
    const semverVersions = VersionUtils.filterSemverVersions(httpResponse.data)
      .toSorted(VersionUtils.compareVersionsAndBuilds);

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
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(httpResponse),
      type: semverSpec.type,
      resolved,
      suggestions,
    };
  }

}
