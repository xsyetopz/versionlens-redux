import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import type { GetSuggestionProvider } from '#domain/useCases';
import { OnTextDocumentChange } from '#extension/events';
import type { ContextState, VersionLensState } from '#extension/state';
import { test } from 'mocha-ui-esm';
import { anything, instance, mock, verify, when } from 'ts-mockito';
import type {
  TextDocument,
  TextDocumentChangeEvent,
  TextDocumentChangeReason
} from 'vscode';

type TestContext = {
  mockState: VersionLensState
  mockGetSuggestionProvider: GetSuggestionProvider
  mockLogger: ILogger
  mockProviderActive: ContextState<string | null>
  testEvent: OnTextDocumentChange
}

export const onTextDocumentChangeTests = {

  [test.title]: OnTextDocumentChange.name,

  beforeEach: function (this: TestContext) {
    this.mockState = mock<VersionLensState>();
    this.mockGetSuggestionProvider = mock<GetSuggestionProvider>();
    this.mockLogger = mock<ILogger>();
    this.mockProviderActive = mock<ContextState<string>>();

    when(this.mockState.providerActive).thenReturn(instance(this.mockProviderActive))

    this.testEvent = new OnTextDocumentChange(
      instance(this.mockGetSuggestionProvider),
      instance(this.mockState),
      instance(this.mockLogger)
    );
  },

  "does not fire event when a provider is not active": async function (this: TestContext) {
    // setup
    const event = mock<TextDocumentChangeEvent>()
    when(this.mockProviderActive.value).thenReturn(null);

    // test
    await this.testEvent.execute(instance(event));

    // verify
    verify(this.mockGetSuggestionProvider.execute(anything())).never();
  },

  "case $i: fires event when a change occurs": [
    [1, []],
    [2, []],
    [undefined, [1]],
    async function (this: TestContext, testReason: TextDocumentChangeReason, testChanges: any[]) {
      const testFilename = 'some-file.txt';
      const testDocText = 'test text';
      const mockChangeEvent = mock<TextDocumentChangeEvent>();
      const mockDocument = mock<TextDocument>();
      const mockProvider = mock<ISuggestionProvider>();

      when(this.mockProviderActive.value).thenReturn('test');
      when(mockChangeEvent.document).thenReturn(instance(mockDocument));
      when(mockDocument.fileName).thenReturn(testFilename);
      when(mockDocument.getText()).thenReturn(testDocText);
      when(this.mockGetSuggestionProvider.execute(testFilename))
        .thenReturn(instance(mockProvider));

      const mockFireEvent = mock<OnTextDocumentChange>();
      this.testEvent.fire = instance(mockFireEvent).fire;

      when(mockChangeEvent.contentChanges).thenReturn(testChanges);
      when(mockChangeEvent.reason).thenReturn(testReason);

      // test
      await this.testEvent.execute(instance(mockChangeEvent));

      // verify
      verify(mockDocument.getText()).once();
      verify(mockFireEvent.fire(instance(mockProvider), anything(), testDocText)).once();
    }
  ],
};