import { type UrlAuthenticationData, AuthenticationScheme } from '#extension/authorization';
import { URL } from 'node:url';

export function createCustomProviderId(scheme: AuthenticationScheme, url: string) {
  switch (scheme) {
    case AuthenticationScheme.Basic:
      return `(Basic Auth) ${url}`;
    case AuthenticationScheme.Custom:
      return `(Custom Auth) ${url}`;
    default:
      throw new Error("Authentication scheme not defined");
  }
}

export function createUrlAuthData(
  url: string,
  id: string,
  label: string,
  scheme: AuthenticationScheme,
  isCustomProvider: boolean
): UrlAuthenticationData {
  const parsedUrl = new URL(url);
  return {
    url,
    protocol: parsedUrl.protocol,
    id,
    label,
    scheme,
    isCustomProvider
  };
}

export function createEmptyUrlAuthData(url: string): UrlAuthenticationData {
  const parsedUrl = new URL(url);
  return {
    url,
    protocol: parsedUrl.protocol,
    id: null,
    label: null,
    scheme: AuthenticationScheme.NotSet,
    isCustomProvider: false
  };
}