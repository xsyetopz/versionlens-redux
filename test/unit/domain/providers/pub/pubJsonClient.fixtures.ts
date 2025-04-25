export default {
  test: {
    versions: [
      {
        version: "0.1.0",
      },
      {
        version: "0.1.1",
      },
      {
        version: "0.1.2",
      },
      {
        version: "0.1.3",
        retracted: true,
      },
      {
        version: "1.0.0-dev.1"
      },
    ]
  },
  expected: {
    "versions": [
      "0.1.0",
      "0.1.1",
      "0.1.2",
      "1.0.0-dev.1"
    ]
  }
}