import {
  type PackageSuggestion,
  PackageStatusFactory,
  SuggestionStatusText,
  UpdateableFactory
} from '#domain/packages';
import type { DockerRepository } from '#domain/providers/docker';

export default {
  node: {
    test: <DockerRepository[]>[
      {
        "name": "latest",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "current-bookworm",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "current",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "bookworm",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23.11.0-bookworm",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23.11.0",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23.11-bookworm",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23.11",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23-bookworm",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "23",
        "digest": "sha256:c5bfe90b30e795ec57bcc0040065ca6f284af84a1dafd22a207bd6b48c39ce01"
      },
      {
        "name": "22.4.3",
        "digest": "sha256:222222222"
      },
      {
        "name": "22.4",
        "digest": "sha256:222222222"
      },
      {
        "name": "22-bookworm",
        "digest": "sha256:222222222"
      },
      {
        "name": "22",
        "digest": "sha256:222222222"
      },
      {
        "name": "21.0.0",
        "digest": "sha256:111111111"
      },
      {
        "name": "21.0",
        "digest": "sha256:111111111"
      },
    ],
    expectLatestStatusWithBuildSuggestions: <PackageSuggestion[]>[
      PackageStatusFactory.createMatchesLatestStatus('23.11.0'),
      UpdateableFactory.createBuildUpdateable('latest,23,23-bookworm,23.11,23.11-bookworm,23.11.0,23.11.0-bookworm,bookworm,current,current-bookworm')
    ],
    expectFixedWithSuggestions: <PackageSuggestion[]>[
      PackageStatusFactory.createFixedStatus('22.4.3'),
      UpdateableFactory.createLatestUpdateable('23-bookworm'),
      UpdateableFactory.createBuildUpdateable('22,22-bookworm,22.4,22.4.3')
    ],
    expectNoMatchWithSuggestions: <PackageSuggestion[]>[
      PackageStatusFactory.createNoMatchStatus(),
      UpdateableFactory.createLatestUpdateable('23'),
      UpdateableFactory.createNextMaxUpdateable('22', SuggestionStatusText.UpdateMajor),
    ]
  },
  mssql: {
    test: [
      {
        "name": "2022-RTM-CU2-ubuntu-20.04",
        "digest": "sha256:7dfe6bf43d18d0cf929e1dc40d0c1fccce02e7cca1ed9e277e35b6815108d17b",
      },
      {
        "name": "2022-RTM-GDR1-ubuntu-20.04",
        "digest": "sha256:9b28787ba6552caa53d2dbcc21a3526363fb34ec1ddf0d50dcdea810cdeaa71d",
      },
      {
        "name": "2022-RTM-ubuntu-20.04",
        "digest": "sha256:7c61aeefa1c8eb55bccfa8d536a283ec922c486c7688e51f193b84c5f0aa3768",
      },
      {
        "name": "2022-latest",
        "digest": "sha256:ea73825f3d88a23c355ac2f9fdc6bd960fec90171c12c572109b36a558f77bb8",
      },
      {
        "name": "2022-preview-ubuntu-22.04",
        "digest": "sha256:6a9e9c9b3caace51857d12ba4c363fd1f94ef7d5a5b2062b102c5be8d51b2776",
      },
      {
        "name": "latest",
        "digest": "sha256:ea73825f3d88a23c355ac2f9fdc6bd960fec90171c12c572109b36a558f77bb8",
      },
      {
        "name": "latest-ubuntu",
        "digest": "sha256:1bbf3b11687ce4d97eb5e6b6e61ccc500d0eff92f92e51893112a3fc665ce7b7",
      }
    ],
    expectLatestStatusWithBuildSuggestions: <PackageSuggestion[]>[
      PackageStatusFactory.createMatchesLatestStatus('latest'),
      UpdateableFactory.createBuildUpdateable('latest,2022-RTM-CU2-ubuntu-20.04,2022-RTM-GDR1-ubuntu-20.04,2022-RTM-ubuntu-20.04,2022-latest,2022-preview-ubuntu-22.04')
    ],
    expectNoMatchWithLatestSuggestion: <PackageSuggestion[]>[
      PackageStatusFactory.createNoMatchStatus(),
      UpdateableFactory.createLatestUpdateable('latest')
    ]
  }
}