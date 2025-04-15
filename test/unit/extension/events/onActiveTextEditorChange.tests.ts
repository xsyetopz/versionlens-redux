import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import type { GetSuggestionProvider } from '#domain/useCases';
import { OnActiveTextEditorChange } from '#extension/events';
import type { ContextState, VersionLensState } from '#extension/state';
import { test } from 'mocha-ui-esm';
import { anything, instance, mock, verify, when } from 'ts-mockito';
import type { TextDocument, TextEditor, Uri } from 'vscode';

type TestContext = {
  mockState: VersionLensState
  mockGetSuggestionProvider: GetSuggestionProvider
  mockLogger: ILogger
  mockActiveState: ContextState<string | null>
  mockTextEditor: TextEditor
  mockDocument: TextDocument
  mockUri: Uri
  testEvent: OnActiveTextEditorChange
}

export const onActiveTextEditorChangeTests = {

  [test.title]: OnActiveTextEditorChange.name,

  beforeEach: function (this: TestContext) {
    this.mockState = mock<VersionLensState>();
    this.mockGetSuggestionProvider = mock<GetSuggestionProvider>();
    this.mockLogger = mock<ILogger>();
    this.mockTextEditor = mock<TextEditor>();
    this.mockDocument = mock<TextDocument>();
    this.mockUri = mock<Uri>();
    this.mockActiveState = mock<ContextState<string | null>>();

    when(this.mockState.providerActive).thenReturn(instance(this.mockActiveState))
    when(this.mockUri.scheme).thenReturn('file');
    when(this.mockDocument.uri).thenReturn(instance(this.mockUri));
    when(this.mockTextEditor.document).thenReturn(instance(this.mockDocument));
    when(this.mockDocument.fileName).thenReturn('/some/file/path');

    this.testEvent = new OnActiveTextEditorChange(
      instance(this.mockState),
      instance(this.mockGetSuggestionProvider),
      instance(this.mockLogger)
    );
  },

  "disable icons when TextEditor is $1": [
    undefined,
    null,
    async function (this: TestContext, testTextEditor: TextEditor) {
      // test
      await this.testEvent.execute(testTextEditor);

      // verify
      verify(this.mockActiveState.change(null)).once();
      verify(this.mockGetSuggestionProvider.execute(anything())).never();
    }
  ],

  "disable icons when TextEditor.document.uri is not a file://":
    async function (this: TestContext) {
      when(this.mockUri.scheme).thenReturn('test');

      // test
      await this.testEvent.execute(instance(this.mockTextEditor));

      // verify
      verify(this.mockActiveState.change(null)).once();
      verify(this.mockGetSuggestionProvider.execute(anything())).never();
    },

  "disable icons when no suggestion provider found": async function (this: TestContext) {
    const testFilePath = '/some/file/path';

    // test
    await this.testEvent.execute(instance(this.mockTextEditor));

    // verify
    verify(this.mockGetSuggestionProvider.execute(testFilePath)).once();
    verify(this.mockActiveState.change(null)).once();
  },

  "updates active provider and fires event":
    async function (this: TestContext) {
      const testFilePath = '/some/file/path';
      const testProviderName = 'testProvider';
      const mockProvider = mock<ISuggestionProvider>();
      const testProvider = instance(mockProvider);

      when(mockProvider.name).thenReturn(testProviderName);
      when(this.mockGetSuggestionProvider.execute(testFilePath))
        .thenReturn(instance(mockProvider));

      const mockFireEvent = mock<OnActiveTextEditorChange>();
      this.testEvent.fire = instance(mockFireEvent).fire;

      // test
      await this.testEvent.execute(instance(this.mockTextEditor));

      // verify
      verify(this.mockGetSuggestionProvider.execute(testFilePath)).once();
      verify(this.mockActiveState.change(testProviderName)).once();
      verify(mockFireEvent.fire(testProvider, instance(this.mockDocument))).once();
    }

};