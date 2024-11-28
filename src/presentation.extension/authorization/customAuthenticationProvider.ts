import {
  type AuthenticationInteractions,
  type IVsCodeAuthentication,
  AuthenticationScheme,
  createCustomProviderId
} from '#extension/authorization';
import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';
import {
  type AuthenticationProvider,
  type AuthenticationProviderSessionOptions,
  type AuthenticationProviderAuthenticationSessionsChangeEvent as AuthenticationProviderSessionsChangeEvent,
  type AuthenticationSession,
  type AuthenticationSessionAccountInformation,
  type AuthenticationSessionsChangeEvent,
  type Event,
  type SecretStorage,
  type SecretStorageChangeEvent,
  Disposable,
  EventEmitter
} from 'vscode';

export class CustomAuthenticationProvider implements AuthenticationProvider, Disposable {

  constructor(
    public scheme: AuthenticationScheme,
    public url: string,
    readonly secretStorage: SecretStorage,
    readonly interactions: AuthenticationInteractions,
    readonly authentication: IVsCodeAuthentication
  ) {
    throwNotStringOrEmpty('scheme', scheme);
    throwNotStringOrEmpty('url', url);
    throwUndefinedOrNull('secretStorage', secretStorage);
    throwUndefinedOrNull('interactions', interactions);
    throwUndefinedOrNull('authentication', authentication);

    this.id = createCustomProviderId(scheme, url);

    switch (scheme) {
      case AuthenticationScheme.Basic:
        this.consentLabel = `basic authentication for ${this.url}`;
        break;
      default:
        this.consentLabel = `custom authentication for ${this.url}`;
        break;
    }
  }

  public id: string;

  public consentLabel: string;

  private _onDidChangeSessions = new EventEmitter<AuthenticationProviderSessionsChangeEvent>();

  private currentToken: Promise<string | undefined> | undefined;

  private disposable: Disposable | undefined;

  get onDidChangeSessions(): Event<AuthenticationProviderSessionsChangeEvent> {
    return this._onDidChangeSessions.event;
  }

  private ensureInitialized(): void {
    if (this.disposable === undefined) {
      this.cacheTokenFromStorage();
      this.disposable = Disposable.from(
        this.secretStorage.onDidChange(this.onSecretDidChange, this),
        this.authentication.onDidChangeSessions(this.onAuthDidChangeSessions, this)
      );
    }
  }

  async getSessions(): Promise<AuthenticationSession[]> {
    this.ensureInitialized();
    const token = await this.cacheTokenFromStorage();
    return token ? [new CustomAuthenticationSession(this.id, token)] : [];
  }

  async createSession(
    scopes: readonly string[],
    options: AuthenticationProviderSessionOptions
  ): Promise<AuthenticationSession> {
    this.ensureInitialized();

    // get the scheme authentication value
    let token: string;
    if (this.scheme === AuthenticationScheme.Basic)
      token = await this.interactions.enterBasicAuthDetails(this.url);
    else
      token = await this.interactions.enterRawAuthDetails(this.url);

    if (token === undefined) return Promise.reject('Authentication was cancelled');

    // store the token
    await this.secretStorage.store(this.id, token);

    return new CustomAuthenticationSession(this.id, token);
  }

  async removeSession(sessionId: string): Promise<void> {
    const token = await this.currentToken;
    if (!token) return;

    await this.secretStorage.delete(this.id);
    this._onDidChangeSessions.fire({
      added: [],
      changed: [],
      removed: [new CustomAuthenticationSession(this.id, token)]
    });
  }

  dispose(): void {
    this.disposable?.dispose();
  }

  private cacheTokenFromStorage(): Promise<string | undefined> {
    this.currentToken = this.secretStorage.get(this.id) as Promise<string | undefined>;
    return this.currentToken;
  }

  // This is a crucial function that handles whether or not the token has changed in
  // a different window of VS Code and sends the necessary event if it has.
  private async checkForUpdates(): Promise<void> {
    const added: AuthenticationSession[] = [];
    const removed: AuthenticationSession[] = [];
    const changed: AuthenticationSession[] = [];

    const previousToken = await this.currentToken;
    const session = (await this.getSessions())[0];

    if (session?.accessToken && !previousToken)
      added.push(session);
    else if (!session?.accessToken && previousToken)
      removed.push(session);
    else if (session?.accessToken !== previousToken)
      changed.push(session);
    else
      return;

    void this.cacheTokenFromStorage();
    this._onDidChangeSessions.fire({ added, removed, changed });
  }

  private onSecretDidChange(e: SecretStorageChangeEvent) {
    if (e.key === this.id) {
      this.checkForUpdates();
    }
  }

  private onAuthDidChangeSessions(e: AuthenticationSessionsChangeEvent) {
    if (e.provider.id === this.id) {
      this.checkForUpdates();
    }
  }

}

class CustomAuthenticationSession implements AuthenticationSession {

  constructor(public readonly id: string, public readonly accessToken: string) {
    this.account = { id, label: id };
  }

  readonly account: AuthenticationSessionAccountInformation;

  readonly scopes: readonly string[] = [];
}