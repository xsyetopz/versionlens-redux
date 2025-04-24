export default {
  get: {
    test: {
      "versions": [
        "1.4.1",
        "1.4.2",
        "1.4.3",
        "1.4.4",
        "1.5.0",
        "1.6.2",
      ],
    },
    expected: {
      versions: [
        "1.4.1",
        "1.4.2",
        "1.4.3",
        "1.4.4",
        "1.5.0",
        "1.6.2",
      ],
    }
  },
  resource: {
    "resources": [
      {
        "@id": "https://unit-test-search-ussc.nuget.org/query",
        "@type": "SearchQueryService",
      },
      {
        "@id": "https://api.nuget.org/v3-flatcontainer1/",
        "@type": "PackageBaseAddress",
      },
      {
        "@id": "https://api.nuget.org/v3-flatcontainer2/",
        "@type": "PackageBaseAddress/3.0.0",
      },
      {
        "@id": "https://unit-test-usnc.nuget.org/",
        "@type": "SearchGalleryQueryService/3.0.0-rc",
      }
    ]
  }
}