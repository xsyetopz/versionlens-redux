import { ILogger } from '#domain/logging';
import { TPackageClientRequest, TPackageClientResponse } from "#domain/packages";
import { IProviderConfig } from 'domain/providers';

export interface IPackageClient<TClientData> {

  logger: ILogger;

  config: IProviderConfig,

  fetchPackage: (request: TPackageClientRequest<TClientData>)
    => Promise<TPackageClientResponse>;

}