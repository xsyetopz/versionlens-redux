import type { ILogger } from '#domain/logging';
import {
  type IPackageClient,
  type PackageClientRequest,
  type PackageClientResponse,
  ClientResponseFactory
} from '#domain/packages';
import { DenoConfig, JsrClient } from '#domain/providers/deno';
import { type NpaSpec, type TNpmClientData, NpmPackageClient } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';
import npa from 'npm-package-arg';

export class DenoClient implements IPackageClient<TNpmClientData> {

  constructor(
    readonly config: DenoConfig,
    readonly jsrClient: JsrClient,
    readonly npmClient: NpmPackageClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsrClient", jsrClient);
    throwUndefinedOrNull("npmClient", npmClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchPackage(request: PackageClientRequest<TNpmClientData>): Promise<PackageClientResponse> {
    const requestedPackage = request.parsedDependency.package;
    const isDenoJsr = requestedPackage.version.startsWith('jsr:');
    const isDenoNpm = requestedPackage.version.startsWith('npm:');
    if (!isDenoJsr && !isDenoNpm) return ClientResponseFactory.createNoSuggestions();
    if (isDenoNpm) return this.npmClient.fetchPackage(request);

    const npaSpec = npa.resolve(
      requestedPackage.name,
      requestedPackage.version.replaceAll('jsr:', 'npm:'),
      requestedPackage.path
    ) as NpaSpec;

    return this.jsrClient.fetchPackage(npaSpec);
  }

}