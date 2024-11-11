import { ClientResponseSource } from '#domain/clients';
import { ILogger } from '#domain/logging';
import {
  ClientResponseFactory,
  IPackageClient,
  PackageSourceType,
  PackageStatusFactory,
  TPackageClientRequest,
  TPackageClientResponse,
  TPackageSuggestion,
  UpdateableFactory
} from '#domain/packages';
import {
  GitHubClient,
  NpaSpec,
  NpaTypes,
  NpmConfig,
  NpmRegistryClient,
  convertNpmErrorToResponse
} from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import npa from 'npm-package-arg';

export class NpmPackageClient implements IPackageClient<any> {

  constructor(
    readonly config: NpmConfig,
    readonly npmRegistryClient: NpmRegistryClient,
    readonly githubClient: GitHubClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("npmRegistryClient", npmRegistryClient);
    throwUndefinedOrNull("githubClient", githubClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: TPackageClientRequest<any>): Promise<TPackageClientResponse> {
    let source: PackageSourceType;

    try {
      const requestedPackage = request.parsedDependency.package;

      const npaSpec = npa.resolve(
        requestedPackage.name,
        requestedPackage.version,
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
        return ClientResponseFactory.createDirectoryFromFileProtocol(requestedPackage);
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
      return await this.npmRegistryClient.fetchPackage(request, npaSpec);

    } catch (response) {
      this.logger.debug("Caught exception from %s: %O", source, response);

      if (!response.data) {
        response = convertNpmErrorToResponse(
          response,
          ClientResponseSource.remote
        );
      }

      const status = response.status &&
        !Number.isInteger(response.status) &&
        response.status.startsWith('E') ?
        response.status.substr(1) :
        response.status;

      let suggestions: Array<TPackageSuggestion>;

      if (status == 'CONNREFUSED')
        suggestions = [PackageStatusFactory.createConnectionRefusedStatus()];
      else if (status == 'CONNRESET')
        suggestions = [PackageStatusFactory.createConnectionResetStatus()];
      else if (status == 'UNSUPPORTEDPROTOCOL' || response.data == 'Not implemented yet')
        suggestions = [PackageStatusFactory.createNotSupportedStatus()];
      else if (status == 'INVALIDTAGNAME' || response.data.includes('Invalid comparator:'))
        suggestions = [
          PackageStatusFactory.createInvalidStatus(''),
          UpdateableFactory.createLatestUpdateable('latest')
        ];
      else if (status == 'INVALIDPACKAGENAME')
        suggestions = [
          PackageStatusFactory.createInvalidStatus('')
        ];
      else if (status == 128)
        suggestions = [PackageStatusFactory.createNotFoundStatus()]
      else
        suggestions = [PackageStatusFactory.createFromHttpStatus(status)];

      if (suggestions === null) throw response;

      return ClientResponseFactory.create(
        source,
        ClientResponseFactory.createResponseStatus(response.source, response.status),
        suggestions
      );

    };

  }

}