import type { IExpiryCache } from '#domain/caching';
import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type INpmRegistry,
  type NpaSpec,
  type NpmClientData,
  type NpmRegistryClientResponse,
  NpmConfig
} from '#domain/providers/npm';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package data from an NPM registry.
 */
export class NpmRegistryClient {

  /**
   * Initializes a new instance of the NpmRegistryClient class.
   * @param npmRegistryFetch The low-level NPM registry fetcher.
   * @param config The configuration for the NPM provider.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly npmRegistryFetch: INpmRegistry,
    readonly config: NpmConfig,
    readonly requestCache: IExpiryCache<NpmRegistryClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('npmRegistryFetch', npmRegistryFetch);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches the packument for a package from the registry.
   * @param npaSpec The NPM package specification.
   * @param clientData The NPM client data including registry URL and SSL options.
   * @returns A promise resolving to the NPM registry client response.
   */
  async get(npaSpec: NpaSpec, clientData: NpmClientData): Promise<NpmRegistryClientResponse> {
    try {
      const registry = this.npmRegistryFetch.pickRegistry(npaSpec, clientData);
      const url = `${ensureEndSlash(registry)}${npaSpec.escapedName}`;
      // check cache
      const cached = this.requestCache.get(url, this.config.caching.duration);
      if (cached) return { ...cached, source: ClientResponseSource.cache };
      // fetch
      this.logger.debug(
        "url: {url}, strict-ssl: {strictSSL}, proxy: {proxy}, https-proxy: {httpsProxy}",
        new URL(url),
        clientData.strictSSL,
        clientData.proxy ? new URL(clientData.proxy) : '',
        clientData.httpsProxy ? new URL(clientData.httpsProxy) : ''
      );
      const registryResponse = await this.npmRegistryFetch.json(url, clientData);
      // reduce
      const result = {
        status: 200,
        source: ClientResponseSource.remote,
        data: {
          ['dist-tags']: registryResponse['dist-tags'] ?? {},
          versions: Object.keys(registryResponse.versions ?? {})
        }
      } as NpmRegistryClientResponse;
      // cache and return
      return this.requestCache.set(url, result);
    } catch (error) {
      const result: NpmRegistryClientResponse = {
        source: ClientResponseSource.remote,
        status: error.code,
        data: error.message,
        rejected: true
      };
      throw result;
    }
  }

}