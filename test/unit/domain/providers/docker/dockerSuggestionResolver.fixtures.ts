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
    expected1: <PackageSuggestion[]>[
      PackageStatusFactory.createMatchesLatestStatus('23.11.0'),
      UpdateableFactory.createBuildUpdateable('latest,23,23-bookworm,23.11,23.11-bookworm,23.11.0,23.11.0-bookworm,bookworm,current,current-bookworm')
    ],
    expected2: <PackageSuggestion[]>[
      PackageStatusFactory.createFixedStatus('22.4.3'),
      UpdateableFactory.createLatestUpdateable('23-bookworm'),
      UpdateableFactory.createBuildUpdateable('22,22-bookworm,22.4,22.4.3')
    ],
    expected3: <PackageSuggestion[]>[
      PackageStatusFactory.createNoMatchStatus(),
      UpdateableFactory.createLatestUpdateable('23'),
      UpdateableFactory.createNextMaxUpdateable('22', SuggestionStatusText.UpdateMajor),
    ]
  },
}