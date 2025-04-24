import { PackagistPackagesResult } from '#domain/providers/composer';

export default {
  test: {
    packages: {
      "test-package-name": [
        {
          time: "2018-09-10T07:42:53+00:00",
          version: "v3.1.3"
        },
        {
          time: "2018-09-04T12:29:52+00:00",
          version: "v3.1.2"
        },
        {
          time: "2018-08-21T13:25:08+00:00",
          version: "v3.1.1"
        },
        {
          time: "2018-07-16T10:42:41+00:00",
          version: "v3.1.0"
        },
        {
          time: "2018-03-12T08:29:18+00:00",
          version: "v3.0.1"
        },
      ]
    }
  },
  expected: <PackagistPackagesResult>{
    packages: {
      "test-package-name": [
        {
          version: "v3.1.3",
        },
        {
          version: "v3.1.2",
        },
        {
          version: "v3.1.1",
        },
        {
          version: "v3.1.0",
        },
        {
          version: "v3.0.1",
        },
      ]
    }
  }
}