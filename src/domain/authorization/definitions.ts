export interface IAuthorizer {
  getConsent(url: string): Promise<boolean>;
  retryCredentials(url: string): Promise<boolean>;
  urlHasAuthConsent(url: string): boolean;
  getToken(url: string): Promise<string | undefined>;
}