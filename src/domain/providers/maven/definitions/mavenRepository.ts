import { RegistryProtocols } from '#domain/utils';

export type MavenRepository = {
  url: string,
  protocol: RegistryProtocols
}