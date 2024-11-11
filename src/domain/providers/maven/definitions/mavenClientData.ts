import { MavenRepository } from '#domain/providers/maven'

export type MavenClientData = {
  repositories: Array<MavenRepository>
}