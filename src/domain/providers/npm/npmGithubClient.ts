import type { GitHubJsonClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientResponse,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  UpdateableFactory,
  VersionUtils,
  createSuggestions
} from '#domain/packages';
import { type NpaSpec, NpmConfig } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import semver from 'semver';

/**
 * Client for fetching package version data for NPM dependencies hosted on GitHub.
 */
export class NpmGitHubClient {

  /**
   * Initializes a new instance of the NpmGitHubClient class.
   * @param config The configuration for the NPM provider.
   * @param githubClient The GitHub JSON client.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: NpmConfig,
    readonly githubClient: GitHubJsonClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("githubClient", githubClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Fetches data for a GitHub-hosted NPM package, resolving tags or commits as appropriate.
   * @param npaSpec The NPM package specification.
   * @returns A promise resolving to the package client response.
   */
  fetchGithub(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    const { validRange } = semver;

    if (npaSpec.gitRange) {
      // we have a semver:x.x.x
      return this.fetchTags(npaSpec);
    }

    if (validRange(npaSpec.gitCommittish, VersionUtils.loosePrereleases)) {
      // we have a #x.x.x
      npaSpec.gitRange = npaSpec.gitCommittish;
      return this.fetchTags(npaSpec);
    }

    // we have a #commit
    return this.fetchCommits(npaSpec);
  }

  /**
   * Fetches tags for a GitHub repository and maps them to semver suggestions.
   * @param npaSpec The NPM package specification.
   * @returns A promise resolving to the package client response.
   */
  async fetchTags(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    const { user, project } = npaSpec.hosted;
    const jsonResponse = await this.githubClient.getTags(user, project)

    // extract versions
    const allVersions = VersionUtils.filterSemverVersions(jsonResponse.data)
      .sort(VersionUtils.compareVersionsAndBuilds);

    const source: PackageSourceType = PackageSourceType.Github;

    const type: PackageVersionType = npaSpec.gitRange
      ? PackageVersionType.Range
      : PackageVersionType.Version;

    const versionRange = npaSpec.gitRange;

    const resolved = {
      name: project,
      version: versionRange,
    };

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      allVersions,
      this.config.prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    return {
      source,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type,
      resolved,
      suggestions
    };
  }

  /**
   * Fetches commits for a GitHub repository and identifies if the current committish is latest.
   * @param npaSpec The NPM package specification.
   * @returns A promise resolving to the package client response.
   */
  async fetchCommits(npaSpec: NpaSpec): Promise<PackageClientResponse> {
    const { user, project } = npaSpec.hosted;

    const jsonResponse = await this.githubClient.getCommits(user, project);

    const commits = jsonResponse.data.toReversed();

    const source: PackageSourceType = PackageSourceType.Github;

    const type = PackageVersionType.Committish;

    const versionRange = npaSpec.gitCommittish;

    if (commits.length === 0) {
      // no commits found
      return ClientResponseFactory.create(
        PackageSourceType.Github,
        jsonResponse,
        [PackageStatusFactory.createNotFoundStatus()]
      )
    }

    const commitIndex = commits.findIndex(
      commit => commit.indexOf(versionRange) > -1
    );

    const latestCommit = commits[commits.length - 1];

    const noMatch = commitIndex === -1;

    const isLatest = versionRange === latestCommit;

    const resolved = {
      name: project,
      version: versionRange,
    };

    const suggestions = [];

    if (noMatch) {
      suggestions.push(
        PackageStatusFactory.createNoMatchStatus(),
        UpdateableFactory.createLatestUpdateable(latestCommit)
      );
    } else if (isLatest) {
      suggestions.push(
        PackageStatusFactory.createMatchesLatestStatus(versionRange)
      );
    } else if (commitIndex > 0) {
      suggestions.push(
        PackageStatusFactory.createFixedStatus(versionRange),
        UpdateableFactory.createLatestUpdateable(latestCommit)
      );
    }

    return {
      source,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type,
      resolved,
      suggestions,
      gitSpec: npaSpec.saveSpec
    };
  }

}