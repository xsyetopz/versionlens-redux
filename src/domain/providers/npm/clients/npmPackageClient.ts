import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type PackageSuggestion,
  type TPackageClientRequest,
  type TPackageClientResponse,
  ClientResponseFactory,
  PackageSourceType,
  PackageStatusFactory
} from '#domain/packages';
import {
  type NpaSpec,
  type TNpmClientData,
  GitHubClient,
  JsrClient,
  NpaTypes,
  NpmConfig,
  NpmRegistryClient,
  convertNpmErrorToResponse,
  createNpmSuggestionFromErrorCode
} from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import npa from 'npm-package-arg';

export class NpmPackageClient implements IPackageClient<TNpmClientData> {

  constructor(
    readonly config: NpmConfig,
    readonly npmRegistryClient: NpmRegistryClient,
    readonly githubClient: GitHubClient,
    readonly jsrClient: JsrClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("npmRegistryClient", npmRegistryClient);
    throwUndefinedOrNull("githubClient", githubClient);
    throwUndefinedOrNull("jsrClient", jsrClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: TPackageClientRequest<TNpmClientData>): Promise<TPackageClientResponse> {
    let source: PackageSourceType;

    try {
      const requestedPackage = request.parsedDependency.package;
      const isDeno = requestedPackage.path.endsWith('deno.json');
      const isDenoJsr = isDeno && requestedPackage.version.startsWith('jsr:');
      const isDenoNpm = isDeno && requestedPackage.version.startsWith('npm:');
      if (isDeno && !isDenoJsr && !isDenoNpm) return ClientResponseFactory.createNoSuggestions();

      const parseVersion = isDenoJsr
        ? requestedPackage.version.replaceAll('jsr:', 'npm:')
        : requestedPackage.version;

      const npaSpec = npa.resolve(
        requestedPackage.name,
        parseVersion,
        requestedPackage.path
      ) as NpaSpec;

      switch (npaSpec.type) {
        case NpaTypes.Directory:
          source = PackageSourceType.Directory
          break;
        case NpaTypes.File:
          source = PackageSourceType.File
          break;
        case NpaTypes.Git:
          source = PackageSourceType.Github
          break;
        case NpaTypes.Version:
        case NpaTypes.Range:
        case NpaTypes.Remote:
        case NpaTypes.Alias:
        case NpaTypes.Tag:
          source = PackageSourceType.Registry
          break;
      }

      // return if directory or file document
      if (source === PackageSourceType.Directory || source === PackageSourceType.File) {
        return await ClientResponseFactory.createDirectoryFromFileProtocol(requestedPackage);
      }

      if (source === PackageSourceType.Github) {

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
        return await this.githubClient.fetchGithub(npaSpec);
      }

      // otherwise return registry result
      return isDenoJsr
        ? this.jsrClient.fetchPackage(npaSpec)
        : await this.npmRegistryClient.fetchPackage(request, npaSpec);

    } catch (response) {
      this.logger.debug("Caught exception from {source}: {error}", source, response);

      if (!response.data) {
        response = convertNpmErrorToResponse(
          response,
          ClientResponseSource.remote
        );
      }

      const status = response.status
      const statusIsNumber = Number.isInteger(status);
      let suggestions: Array<PackageSuggestion>;

      if (statusIsNumber)
        suggestions = [
          status === 128
            ? PackageStatusFactory.createNotFoundStatus()
            : PackageStatusFactory.createFromHttpStatus(status)
        ];
      else
        suggestions = createNpmSuggestionFromErrorCode(status);

      return ClientResponseFactory.create(
        source,
        ClientResponseFactory.createResponseStatus(response.source, response.status),
        suggestions
      );
    };

  }

}