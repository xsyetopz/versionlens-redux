import assert from 'node:assert';
import { UrlUtils } from '#domain/clients';
import { test } from 'mocha-ui-esm';

export const UrlHelpersTests = {

  [test.title]: "UrlHelpers",

  getProtocolFromUrl: {

    "parses $2 protocols": [
      ['http://test.url.example/path', UrlUtils.RegistryProtocols.http],
      ['https://test.url.example/path', UrlUtils.RegistryProtocols.https],
      (testUrl: string, expectedProtocol: string) => {
        const actual = UrlUtils.getProtocolFromUrl(testUrl)
        assert.equal(actual, expectedProtocol, "Protocol did not match")
      }
    ],

    "parses file protocols for $1": [
      ['d:\\some\\path'],
      ['/d/some/path'],
      (testFolder: string) => {
        const actual = UrlUtils.getProtocolFromUrl(testFolder)
        assert.equal(actual, UrlUtils.RegistryProtocols.file, "Protocol did not match")
      }
    ],

  },

  ensureEndSlash: {

    "appends missing slashes for $1": [
      ['https://test1.url.example', 'https://test1.url.example/'],
      ['https://test2.url.example/', 'https://test2.url.example/'],
      (testUrl: string, expectedUrl: string) => {
        const actual = UrlUtils.ensureEndSlash(testUrl)
        assert.equal(actual, expectedUrl, "End slash did not match")
      },
    ]

  },

};