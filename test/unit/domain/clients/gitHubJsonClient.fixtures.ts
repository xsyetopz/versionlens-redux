export default {
  tags: {
    test: [
      {
        "name": "v2.6.0-rc.1"
      },
      {
        "name": "v2.5.0"
      },
      {
        "name": "v2.5.0-preview.1"
      },
      {
        "name": "v2.4.3"
      },
      {
        "name": "v2.4.2"
      },
      {
        "name": "v2.4.1"
      }
    ],
    expected: [
      "v2.6.0-rc.1",
      "v2.5.0",
      "v2.5.0-preview.1",
      "v2.4.3",
      "v2.4.2",
      "v2.4.1"
    ]
  },
  commits: {
    test: [
      {
        sha: "f099459fd01be79187275ddf47d77a2797188c6a"
      },
      {
        sha: "166c3497967489e61a1d532b79b8fe750fd5ba56"
      },
      {
        sha: "37250168e2ecaab477c962071d2024e89ebb1844"
      },
      {
        sha: "6a3fb5a4dec4588b746ac3bf14d0704498e7b948"
      },
      {
        sha: "df4d9435a320c0345ff2930ec71a007f3a320211"
      },
    ],
    expected: [
      "f099459",
      "166c349",
      "3725016",
      "6a3fb5a",
      "df4d943",
    ]
  }
}