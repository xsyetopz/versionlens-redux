import { PackageStatusFactory, PackageSuggestion, UpdateableFactory } from '#domain/packages';
import { DockerApiTagResult } from '#domain/providers/docker';

export default {
  test: <DockerApiTagResult[]>[
    {
      "name": "latest",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "current-bookworm",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "current",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "bookworm",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23.11.0-bookworm",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23.11.0",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23.11-bookworm",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23.11",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23-bookworm",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    },
    {
      "name": "23",
      "tag_status": "active",
      "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
    }
  ],
  expected: <PackageSuggestion[]>[
    PackageStatusFactory.createMatchesLatestStatus('23.11.0'),
    UpdateableFactory.createBuildUpdateable('latest,23,23-bookworm,23.11,23.11-bookworm,23.11.0,23.11.0-bookworm,bookworm,current,current-bookworm')
  ]
}