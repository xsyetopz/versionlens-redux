import { DockerApiResponse, DockerApiTagResult } from '#domain/providers/docker';

export default {
  test: <DockerApiResponse<DockerApiTagResult>>{
    "results": [
      {
        "name": "latest",
        "tag_status": "active",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23",
        "tag_status": "active",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "removed",
        "tag_status": "inactive",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "removed",
        "tag_status": "active",
        "digest": undefined
      },
    ]
  },
  expected: <DockerApiTagResult[]>[
    {
      "name": "latest",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    }
  ]
}