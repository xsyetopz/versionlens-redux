import type { AuthenticationInteractions } from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { SecretStorage } from 'vscode';

export abstract class AuthenticationProvider {

  constructor(
    readonly storeKey: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    throwUndefinedOrNull('storeKey', storeKey);
    throwUndefinedOrNull('secretStorage', secretStorage);
    throwUndefinedOrNull('interactions', interactions);
  }

  abstract create(url: string): Promise<boolean>;

  async get(url: string): Promise<string | undefined> {
    return await this.secretStorage.get(this.getKey(url));
  }

  async remove(url: string) {
    await this.secretStorage.delete(this.getKey(url));
  }

  protected getKey(url: string): string { return `${this.storeKey}__${url}`; }
}

export class BasicAuthProvider extends AuthenticationProvider {

  constructor(
    readonly storeKey: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    super(storeKey, secretStorage, interactions);
  }

  async create(url: string): Promise<boolean> {
    const token = await this.interactions.enterBasicAuthDetails(url);
    if (token === undefined) return false;
    await this.secretStorage.store(this.getKey(url), token);
    return true;
  }

}

export class CustomAuthProvider extends AuthenticationProvider {

  constructor(
    readonly storeKey: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    super(storeKey, secretStorage, interactions);
  }

  async create(url: string): Promise<boolean> {
    const token = await this.interactions.enterCustomAuthValue(url);
    if (token === undefined) return false;
    await this.secretStorage.store(this.getKey(url), token);
    return true;
  }

}