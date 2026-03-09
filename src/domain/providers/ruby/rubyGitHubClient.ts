import { GitHubJsonClient } from '#domain/clients';
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
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Shared client for fetching package version data from GitHub.
 */
export class RubyGitHubClient {

  apiUrl: string = 'https://api.github.com';

  /**
   * Initializes a new instance of the GitHubClient class.
   * @param githubJsonClient The JSON HTTP client.
   * @param requestCache The cache for GitHub API requests.
   */
  constructor(readonly githubJsonClient: GitHubJsonClient) {
    throwUndefinedOrNull("githubJsonClient", githubJsonClient);
  }

  /**
   * Fetches tags for a GitHub repository and maps them to semver suggestions.
   * @param user The repository owner.
   * @param project The repository name.
   * @param versionRange The version range to resolve.
   * @param prereleaseTagFilter Optional filter for prerelease tags.
   * @param overrideType Optional type to override the default (Range/Version).
   * @returns A promise resolving to the package client response.
   */
  async fetchTags(
    user: string,
    project: string,
    versionRange: string,
    prereleaseTagFilter: string[] = []
  ): Promise<PackageClientResponse> {
    const jsonResponse = await this.githubJsonClient.getTags(user, project)

    // extract versions
    const allVersions = VersionUtils.filterSemverVersions(jsonResponse.data)
      .sort(VersionUtils.compareVersionsAndBuilds);

    const source: PackageSourceType = PackageSourceType.Github;

    const type: PackageVersionType = versionRange
      ? PackageVersionType.Range
      : PackageVersionType.Version;

    const resolved = {
      name: project,
      version: versionRange,
    };

    // seperate versions to releases and prereleases
    const { releases, prereleases } = VersionUtils.splitReleasesFromArray(
      allVersions,
      prereleaseTagFilter
    );

    // analyse suggestions
    const suggestions = createSuggestions(
      versionRange,
      releases,
      prereleases
    );

    if (releases.includes(versionRange) === false && prereleases.includes(versionRange) === false) {
      suggestions[0] = PackageStatusFactory.createNotFoundStatus()
    }

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
   * @param user The repository owner.
   * @param project The repository name.
   * @param commitSha The commit SHA or reference to resolve.
   * @returns A promise resolving to the package client response.
   */
  async fetchCommits(
    user: string,
    project: string,
    commitSha: string
  ): Promise<PackageClientResponse> {
    const source: PackageSourceType = PackageSourceType.Github;
    const type = PackageVersionType.Committish;
    const resolved = { name: project, version: commitSha };
    const jsonResponse = await this.githubJsonClient.getCommits(user, project);

    const commits = jsonResponse.data.toReversed();
    if (commits.length === 0) {
      return ClientResponseFactory.create(
        PackageSourceType.Github,
        jsonResponse,
        [PackageStatusFactory.createNotFoundStatus()]
      )
    }

    const latestCommit = commits[commits.length - 1];
    const isValidLength = commitSha.length === 7 || commitSha.length === 40;
    const commitIndex = (commitSha !== '' && isValidLength)
      ? commits.findIndex((shortSha: string) => {
        const normalizedShort = shortSha.toLowerCase();
        // If search is 7 chars, they must be equal
        // If search is 40 chars, it must start with the shortSha
        return commitSha.startsWith(normalizedShort);
      })
      : -1;

    const isLatest = isValidLength && commitSha.startsWith(latestCommit);
    const suggestions = [];

    if (commitSha === '' || commitIndex === -1) {
      suggestions.push(
        PackageStatusFactory.createNoMatchStatus(),
        UpdateableFactory.createLatestUpdateable(latestCommit)
      );
    } else if (isLatest) {
      suggestions.push(
        PackageStatusFactory.createMatchesLatestStatus(commitSha.substring(0, 7))
      );
    } else {
      const status = PackageStatusFactory.createFixedStatus(commitSha.substring(0, 7));
      suggestions.push(
        status,
        UpdateableFactory.createLatestUpdateable(latestCommit.substring(0, 7))
      );
    }

    return {
      source,
      responseStatus: ClientResponseFactory.mapStatusFromJsonResponse(jsonResponse),
      type,
      resolved,
      suggestions
    };
  }

}