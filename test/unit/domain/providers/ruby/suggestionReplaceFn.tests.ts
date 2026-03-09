import { SuggestionUpdate } from '#domain/packages';
import { RubyConfig, RubySuggestionProvider, RubySuggestionResolver } from '#domain/providers/ruby';
import { ILogger } from '#domain/logging';
import { equal } from 'node:assert';
import { instance, mock } from 'ts-mockito';
import Fixtures from './suggestionReplaceFn.fixtures';

export const RubySuggestionReplaceFnTests = {

  title: 'RubySuggestionReplaceFn',

  suggestionReplaceFn: {

    "case $i: correctly replaces versions": [
      ...Fixtures.suggestionReplaceFn,
      (fixture: any) => {
        // setup
        const { suggestion, newVersion, expected } = fixture;
        suggestion.suggestionVersion = newVersion;

        const resolverMock = mock(RubySuggestionResolver);
        const configMock = mock(RubyConfig);
        const loggerMock = mock<ILogger>();

        const cut = new RubySuggestionProvider(
          instance(resolverMock),
          instance(configMock),
          instance(loggerMock)
        );

        // test
        const actual = cut.suggestionReplaceFn(suggestion as SuggestionUpdate, newVersion);

        // assert
        equal(actual, expected);
      }
    ]

  }

}
