import type { IShellClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type MavenConfig, type MavenRepository, extractReposUrlsFromXml } from '#domain/providers/maven';
import { getProtocolFromUrl } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for interacting with the Maven CLI.
 */
export class MvnCli {

  /**
   * Initializes a new instance of the MvnCli class.
   * @param config The configuration for the Maven provider.
   * @param shellClient The shell client for executing commands.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: MavenConfig,
    readonly shellClient: IShellClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("shellClient", shellClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Fetches the configured Maven repositories using 'mvn help:effective-settings'.
   * @param cwd The current working directory.
   * @returns A promise resolving to an array of Maven repositories.
   */
  async fetchRepositories(cwd: string): Promise<Array<MavenRepository>> {
    let repos: Array<string>;

    try {
      const result = await this.shellClient.request(
        'mvn ',
        ['help:effective-settings'],
        cwd
      );

      const { data } = result;
      if (data.length === 0) return [];

      repos = extractReposUrlsFromXml(data);

    } catch (err) {
      repos = [];
    }

    if (repos.length === 0) {
      // this.config.getDefaultRepository()
      repos.push("https://repo.maven.apache.org/maven2/");
    }

    // parse urls to Array<MavenRepository>
    return repos.map(url => {
      const protocol = getProtocolFromUrl(url);
      return {
        url,
        protocol,
      };
    });
  }

}