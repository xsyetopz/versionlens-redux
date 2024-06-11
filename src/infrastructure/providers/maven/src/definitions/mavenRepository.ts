import { UrlUtils } from '#domain/clients'

export type MavenRepository = {
  url: string,
  protocol: UrlUtils.RegistryProtocols
}
