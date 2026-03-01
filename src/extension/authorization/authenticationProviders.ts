import type { AuthenticationInteractions } from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { SecretStorage } from 'vscode';

/**
 * Base class for authentication providers.
 */
export abstract class AuthenticationProvider {

  /**
   * Initializes a new instance of the AuthenticationProvider class.
   * @param storeKey The key prefix for storage.
   * @param secretStorage The VS Code secret storage.
   * @param interactions The authentication interactions handler.
   */
  constructor(
    readonly storeKey: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    throwUndefinedOrNull('storeKey', storeKey);
    throwUndefinedOrNull('secretStorage', secretStorage);
    throwUndefinedOrNull('interactions', interactions);
  }

  /**
   * Creates a new authentication session for a URL.
   * @param url The URL to authenticate.
   * @returns A promise resolving to true if successful, otherwise false.
   */
  abstract create(url: string): Promise<boolean>;

  /**
   * Gets the stored token for a URL.
   * @param url The authenticated URL.
   * @returns A promise resolving to the token, or undefined if not found.
   */
  async get(url: string): Promise<string | undefined> {
    return await this.secretStorage.get(this.getKey(url));
  }

  /**
   * Removes the stored token for a URL.
   * @param url The authenticated URL.
   */
  async remove(url: string) {
    await this.secretStorage.delete(this.getKey(url));
  }

  /**
   * Generates the unique storage key for a URL.
   * @param url The authenticated URL.
   * @returns The storage key string.
   */
  protected getKey(url: string): string { return `${this.storeKey}__${url}`; }
}

/**
 * Provider for Basic Authentication.
 */
export class BasicAuthProvider extends AuthenticationProvider {

  /**
   * Initializes a new instance of the BasicAuthProvider class.
   */
  constructor(
    readonly storeKey: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    super(storeKey, secretStorage, interactions);
  }

  /**
   * Creates a basic auth token by prompting the user for credentials.
   */
  async create(url: string): Promise<boolean> {
    const token = await this.interactions.enterBasicAuthDetails(url);
    if (token === undefined) return false;
    await this.secretStorage.store(this.getKey(url), token);
    return true;
  }

}

/**
 * Provider for custom authorization values.
 */
export class CustomAuthProvider extends AuthenticationProvider {

  /**
   * Initializes a new instance of the CustomAuthProvider class.
   */
  constructor(
    readonly storeKey: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    super(storeKey, secretStorage, interactions);
  }

  /**
   * Creates a custom auth token by prompting the user for a value.
   */
  async create(url: string): Promise<boolean> {
    const token = await this.interactions.enterCustomAuthValue(url);
    if (token === undefined) return false;
    await this.secretStorage.store(this.getKey(url), token);
    return true;
  }

}