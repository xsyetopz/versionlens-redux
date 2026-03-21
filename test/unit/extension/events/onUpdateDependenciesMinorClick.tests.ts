import {
  PackageDependency,
  PackageResponse,
  PackageSourceType,
  PackageVersionType,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  createSuggestion
} from '#domain/packages';
import { PackageDescriptor } from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider, GetSuggestions } from '#domain/useCases';
import { IContextState, IVersionLensState, VersionLensExtension } from '#extension';
import { OnUpdateDependenciesMinorClick } from '#extension/events';
import { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { test } from 'mocha-ui-esm';
import { anything, instance, mock, verify, when } from 'ts-mockito';
import type { TextEditor, WorkspaceEdit } from 'vscode';

type TestContext = {
  mockExtension: VersionLensExtension
  mockConstruct: IVsCodeConstructFactory
  mockWorkspace: IVsCodeWorkspace
  mockState: IVersionLensState
  mockGetSuggestionProvider: GetSuggestionProvider
  mockGetSuggestions: GetSuggestions
  mockCodeLensReplace: IContextState<boolean>
  mockShowPrereleases: IContextState<boolean>
}

/**
 * Tests for the OnUpdateDependenciesMinorClick event handler.
 */
export const onUpdateDependenciesMinorClickTests = {

  [test.title]: OnUpdateDependenciesMinorClick.name,

  beforeEach: function (this: TestContext) {
    this.mockExtension = mock(VersionLensExtension)
    this.mockConstruct = mock<IVsCodeConstructFactory>()
    this.mockWorkspace = mock<IVsCodeWorkspace>()
    this.mockState = mock<IVersionLensState>()
    this.mockGetSuggestionProvider = mock(GetSuggestionProvider)
    this.mockGetSuggestions = mock(GetSuggestions)
    this.mockCodeLensReplace = mock<IContextState<boolean>>()
    this.mockShowPrereleases = mock<IContextState<boolean>>()

    when(this.mockShowPrereleases.value).thenReturn(false)
    when(this.mockState.showPrereleases).thenReturn(instance(this.mockShowPrereleases))

    when(this.mockCodeLensReplace.value).thenReturn(true)
    when(this.mockState.codeLensReplace).thenReturn(instance(this.mockCodeLensReplace))

    when(this.mockState.increaseBusyState()).thenResolve()
    when(this.mockState.decreaseBusyState()).thenResolve()
    when(this.mockState.enableCodeLensReplace(anything())).thenResolve()
    when(this.mockState.setErrorState()).thenResolve()

    when(this.mockExtension.isWorkspaceMode).thenReturn(false)
  },

  "applies updates only for Updateable and Minor release suggestions": async function (this: TestContext) {
    // setup
    const testFileName = 'package.json';
    const mockProvider = mock<ISuggestionProvider>();
    when(mockProvider.name).thenReturn('test-provider');

    const descriptors = new PackageDescriptor([
      {
        type: 'version',
        version: '1.0.0',
        versionRange: { start: 10, end: 15 },
        versionPrepend: '',
        versionAppend: ''
      }
    ]);

    const testSuggestions: Array<PackageResponse> = [
      // Should be applied (Updateable, Minor, Release)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.UpdateMinor,
          SuggestionCategory.Updateable,
          '1.1.0',
          SuggestionTypes.release
        ),
        parsedDependency: new PackageDependency(
          { name: 'pkg1', version: '1.0.0', path: '' },
          descriptors
        ),
        packageSource: PackageSourceType.Registry,
        type: PackageVersionType.Version,
        order: 0
      },
      // Should NOT be applied (name not Minor)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.UpdateMajor,
          SuggestionCategory.Updateable,
          '2.0.0',
          SuggestionTypes.release
        ),
        parsedDependency: new PackageDependency(
          { name: 'pkg2', version: '1.0.0', path: '' },
          descriptors
        ),
        packageSource: PackageSourceType.Registry,
        type: PackageVersionType.Version,
        order: 0
      }
    ];

    const mockEditor = {
      document: {
        fileName: testFileName,
        uri: { fsPath: testFileName, scheme: 'file' },
        positionAt: (offset: number) => ({ line: 0, character: offset })
      }
    } as TextEditor;

    const mockWorkspaceEdit = mock<WorkspaceEdit>();

    when(this.mockGetSuggestionProvider.execute(testFileName)).thenReturn(instance(mockProvider));
    when(this.mockGetSuggestions.execute(instance(mockProvider), '.', testFileName, false))
      .thenResolve(testSuggestions);
    when(this.mockConstruct.createWorkspaceEdit()).thenReturn(instance(mockWorkspaceEdit));
    when(this.mockConstruct.createRange(anything(), anything())).thenReturn({} as any);

    const testEvent = new OnUpdateDependenciesMinorClick(
      instance(this.mockExtension),
      instance(this.mockConstruct),
      instance(this.mockWorkspace),
      instance(this.mockState),
      instance(this.mockGetSuggestionProvider),
      instance(this.mockGetSuggestions)
    );

    // test
    await testEvent.execute(mockEditor);

    // assert
    verify(mockWorkspaceEdit.replace(anything(), anything(), anything())).once();
    verify(this.mockWorkspace.applyEdit(instance(mockWorkspaceEdit))).once();
  },

  "skips updates for project version suggestions": async function (this: TestContext) {
    // setup
    const testFileName = 'package.json';
    const mockProvider = mock<ISuggestionProvider>();
    when(mockProvider.name).thenReturn('test-provider');

    const descriptors = new PackageDescriptor([
      {
        type: 'version',
        version: '1.0.0',
        versionRange: { start: 10, end: 15 },
        versionPrepend: '',
        versionAppend: ''
      },
      {
        type: 'projectVersion'
      }
    ]);

    const testSuggestions: Array<PackageResponse> = [
      // Project version (Should NOT be applied)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.UpdateMinor,
          SuggestionCategory.Updateable,
          '1.1.0',
          SuggestionTypes.release
        ),
        parsedDependency: new PackageDependency(
          { name: 'my-project', version: '1.0.0', path: '' },
          descriptors
        ),
        packageSource: PackageSourceType.Registry,
        type: PackageVersionType.Version,
        order: 0
      }
    ];

    const mockEditor = {
      document: {
        fileName: testFileName,
        uri: { fsPath: testFileName, scheme: 'file' },
        positionAt: (offset: number) => ({ line: 0, character: offset })
      }
    } as TextEditor;

    const mockWorkspaceEdit = mock<WorkspaceEdit>();

    when(this.mockGetSuggestionProvider.execute(testFileName)).thenReturn(instance(mockProvider));
    when(this.mockGetSuggestions.execute(instance(mockProvider), '.', testFileName, false))
      .thenResolve(testSuggestions);
    when(this.mockConstruct.createWorkspaceEdit()).thenReturn(instance(mockWorkspaceEdit));
    when(this.mockConstruct.createRange(anything(), anything())).thenReturn({} as any);

    const testEvent = new OnUpdateDependenciesMinorClick(
      instance(this.mockExtension),
      instance(this.mockConstruct),
      instance(this.mockWorkspace),
      instance(this.mockState),
      instance(this.mockGetSuggestionProvider),
      instance(this.mockGetSuggestions)
    );

    // test
    await testEvent.execute(mockEditor);

    // assert
    verify(mockWorkspaceEdit.replace(anything(), anything(), anything())).never();
    verify(this.mockWorkspace.applyEdit(instance(mockWorkspaceEdit))).never();
  }

};
