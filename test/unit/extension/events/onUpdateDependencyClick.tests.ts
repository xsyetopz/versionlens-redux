import type { ILogger } from '#domain/logging';
import {
  createPackageResource,
  PackageDependency,
  SuggestionCategory,
  SuggestionTypes
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import type { IContextState, ISuggestionCodeLens, IVersionLensState } from '#extension';
import { OnUpdateDependencyClick } from '#extension/events';
import type { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { test } from 'mocha-ui-esm';
import { deepEqual, equal } from 'node:assert';
import { anything, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  mockConstruct: IVsCodeConstructFactory
  mockWorkspace: IVsCodeWorkspace
  mockState: IVersionLensState
  mockLogger: ILogger
  mockCodeLensReplace: IContextState<boolean>
}

export const onUpdateDependencyClickTests = {

  [test.title]: OnUpdateDependencyClick.name,

  beforeEach: function (this: TestContext) {
    this.mockConstruct = mock<IVsCodeConstructFactory>()
    this.mockWorkspace = mock<IVsCodeWorkspace>()
    this.mockState = mock<IVersionLensState>()
    this.mockLogger = mock<ILogger>()
    this.mockCodeLensReplace = mock<IContextState<boolean>>()
  },

  "prevents replace when disabled": async function (this: TestContext) {
    // setup
    const testEvent = new OnUpdateDependencyClick(
      instance(this.mockConstruct),
      instance(this.mockWorkspace),
      instance(this.mockState),
      instance(this.mockLogger)
    );
    const testCodelens = mock<ISuggestionCodeLens>()

    when(this.mockCodeLensReplace.value).thenReturn(false)
    when(this.mockState.codeLensReplace).thenReturn(instance(this.mockCodeLensReplace))

    // test
    await testEvent.execute(instance(testCodelens));

    // verify
    verify(this.mockState.enableCodeLensReplace(anything())).never();
  },

  "applies edits to '$1' versions": [
    [SuggestionTypes.tag, 'next', 'next'],
    [SuggestionTypes.release, '1.2.3', '2.3.4'],
    async function (this: TestContext, testType: SuggestionTypes, testVersion: string, expectedVersion: string) {
      // setup
      const testDocumentUrl = 'test-url'
      const testReplaceRange = {}
      const testEvent = new OnUpdateDependencyClick(
        instance(this.mockConstruct),
        instance(this.mockWorkspace),
        instance(this.mockState),
        instance(this.mockLogger)
      );
      const mockCodeLens = mock<ISuggestionCodeLens>()
      when(mockCodeLens.documentUrl).thenReturn(testDocumentUrl as any)
      when(mockCodeLens.replaceRange).thenReturn(testReplaceRange as any)
      when(mockCodeLens.packageResponse).thenReturn({
        suggestion: {
          name: 'latest',
          version: testVersion,
          type: testType,
          category: SuggestionCategory.Latest
        },
        providerName: 'test-provider',
        parsedDependency: new PackageDependency(
          createPackageResource('test-name', testVersion, 'test/path'),
          new PackageDescriptor([
            createPackageNameDesc('testPackage1', createTextRange(0, 0)),
            createPackageVersionDesc('1.0.0', createTextRange(1, 1)),
          ])
        ),
        order: 0,
      })
      when(this.mockCodeLensReplace.value).thenReturn(true)
      when(this.mockState.codeLensReplace).thenReturn(instance(this.mockCodeLensReplace))
      const testEdit = {
        replace: function (actualUrl: any, actualRange: any, actualVersion: string) {
          equal(actualUrl, testDocumentUrl)
          deepEqual(actualRange, testReplaceRange)
          equal(actualVersion, expectedVersion)
        }
      }
      when(this.mockConstruct.createWorkspaceEdit()).thenReturn(testEdit as any)
      when(mockCodeLens.replaceVersionFn(anything(), testVersion)).thenReturn(expectedVersion)

      // test
      await testEvent.execute(instance(mockCodeLens));

      // verify
      verify(this.mockState.enableCodeLensReplace(false)).once();
      if (testType !== SuggestionTypes.tag)
        verify(mockCodeLens.replaceVersionFn(anything(), testVersion)).once()
      verify(this.mockConstruct.createWorkspaceEdit()).once();
      verify(this.mockWorkspace.applyEdit(testEdit as any)).once();
    }
  ],

};