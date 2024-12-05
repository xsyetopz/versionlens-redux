import type { AuthenticationInteractions } from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { SecretStorage } from 'vscode';

export abstract class AuthenticationProvider {

  constructor(
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    throwUndefinedOrNull('secretStorage', secretStorage);
    throwUndefinedOrNull('interactions', interactions);
  }

  abstract create(url: string): Promise<boolean>;

  async get(url: string): Promise<string | undefined> {
    return await this.secretStorage.get(url);
  }

  async clear(url: string) {
    await this.secretStorage.store(url, undefined);
  }

}

export class BasicAuthProvider extends AuthenticationProvider {

  constructor(
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    super(secretStorage, interactions);
  }

  async create(url: string): Promise<boolean> {
    const token = await this.interactions.enterBasicAuthDetails(url);
    if (token === undefined) return false;
    await this.secretStorage.store(url, token);
    return true;
  }

}

export class CustomAuthProvider extends AuthenticationProvider {

  constructor(
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions
  ) {
    super(secretStorage, interactions);
  }

  async create(url: string): Promise<boolean> {
    const token = await this.interactions.enterCustomAuthValue(url);
    if (token === undefined) return false;
    await this.secretStorage.store(url, token);
    return true;
  }

}