import type { IJsonHttpClient, JsonClientResponse } from '#domain/clients';
import { throwUndefinedOrNull } from '@esm-test/guards';

const defaultHeaders = {
  accept: 'application\/vnd.github.v3+json'
};

export class GitHubJsonClient {

  constructor(readonly jsonClient: IJsonHttpClient) {
    throwUndefinedOrNull("jsonClient", jsonClient);
  }

  async getTags(user: string, project: string): Promise<JsonClientResponse<string[]>> {
    const tagsRepoUrl = `https://api.github.com/repos/${user}/${project}/tags`;
    const jsonResponse = await this.jsonClient.get(tagsRepoUrl, {}, defaultHeaders);
    const tags = jsonResponse.data ?? [];
    return {
      ...jsonResponse,
      data: tags.map((tag: any) => tag.name)
    };
  }

  async getCommits(user: string, project: string): Promise<JsonClientResponse<string[]>> {
    const commitsRepoUrl = `https://api.github.com/repos/${user}/${project}/commits`;
    const jsonResponse = await this.jsonClient.get(commitsRepoUrl, {}, defaultHeaders);
    const commitInfos = <[]>jsonResponse.data
    const data = commitInfos.map((commit: any) => commit.sha);
    return {
      ...jsonResponse,
      data
    }
  }

}