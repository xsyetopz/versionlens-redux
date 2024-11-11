import { ensureEndSlash, getProtocolFromUrl, RegistryProtocols } from '#domain/utils';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';

export const UrlTests = {

  [test.title]: "Url Utils",

  [getProtocolFromUrl.name]: {

    "parses $2 protocols": [
      ['http://test.url.example/path', RegistryProtocols.http],
      ['https://test.url.example/path', RegistryProtocols.https],
      (testUrl: string, expectedProtocol: string) => {
        const actual = getProtocolFromUrl(testUrl)
        assert.equal(actual, expectedProtocol, "Protocol did not match")
      }
    ],

    "parses file protocols for $1": [
      ['d:\\some\\path'],
      ['/d/some/path'],
      (testFolder: string) => {
        const actual = getProtocolFromUrl(testFolder)
        assert.equal(actual, RegistryProtocols.file, "Protocol did not match")
      }
    ],

  },

  [ensureEndSlash.name]: {

    "appends missing slashes for $1": [
      ['https://test1.url.example', 'https://test1.url.example/'],
      ['https://test2.url.example/', 'https://test2.url.example/'],
      (testUrl: string, expectedUrl: string) => {
        const actual = ensureEndSlash(testUrl)
        assert.equal(actual, expectedUrl, "End slash did not match")
      },
    ]

  },

};