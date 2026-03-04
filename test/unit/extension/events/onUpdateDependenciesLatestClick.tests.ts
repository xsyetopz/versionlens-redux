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
import { OnUpdateDependenciesLatestClick } from '#extension/events';
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
 * Tests for the OnUpdateDependenciesLatestClick event handler.
 * Covers:
 * - Early exits: No editor, codeLensReplace disabled, or no provider found.
 * - Filtering: Only applies updates for 'Updateable' 'Latest' 'Release' suggestions.
 * - Successful execution: Increments busy state, retrieves suggestions, applies WorkspaceEdit with correct versions, and handles range deduplication.
 * - Error handling: Sets error state when suggestions retrieval fails.
 * - Contextual behavior: Correct project path resolution in workspace mode and support for custom suggestion replacement functions.
 */
export const onUpdateDependenciesLatestClickTests = {

  [test.title]: OnUpdateDependenciesLatestClick.name,

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

  "does not update if no text editor is provided": async function (this: TestContext) {
    // setup
    const testEvent = new OnUpdateDependenciesLatestClick(
      instance(this.mockExtension),
      instance(this.mockConstruct),
      instance(this.mockWorkspace),
      instance(this.mockState),
      instance(this.mockGetSuggestionProvider),
      instance(this.mockGetSuggestions)
    );

    // test
    await testEvent.execute(undefined);

    // assert
    verify(this.mockGetSuggestionProvider.execute(anything())).never();
  },

  "does not update if codeLensReplace is disabled": async function (this: TestContext) {
    // setup
    const mockEditor = {
      document: {
        fileName: 'package.json'
      }
    } as TextEditor;

    const testEvent = new OnUpdateDependenciesLatestClick(
      instance(this.mockExtension),
      instance(this.mockConstruct),
      instance(this.mockWorkspace),
      instance(this.mockState),
      instance(this.mockGetSuggestionProvider),
      instance(this.mockGetSuggestions)
    );

    when(this.mockCodeLensReplace.value).thenReturn(false)

    // test
    await testEvent.execute(mockEditor);

    // assert
    verify(this.mockGetSuggestionProvider.execute(anything())).never();
  },

  "does not update if provider is not found": async function (this: TestContext) {
    // setup
    const testFileName = 'package.json';
    const mockEditor = {
      document: {
        fileName: testFileName
      }
    } as TextEditor;

    when(this.mockGetSuggestionProvider.execute(testFileName)).thenReturn(undefined);

    const testEvent = new OnUpdateDependenciesLatestClick(
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
    verify(this.mockGetSuggestionProvider.execute(anything())).once();
    verify(this.mockState.increaseBusyState()).never();
  },

  "applies updates only for Updateable and Latest release suggestions": async function (this: TestContext) {
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
      // Should be applied (Updateable, Latest, Release)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.Latest,
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
      // Should NOT be applied (category not Updateable)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.Latest,
          SuggestionCategory.Latest,
          '1.0.0',
          SuggestionTypes.release
        ),
        parsedDependency: new PackageDependency(
          { name: 'pkg2', version: '1.0.0', path: '' },
          descriptors
        ),
        packageSource: PackageSourceType.Registry,
        type: PackageVersionType.Version,
        order: 0
      },
      // Should NOT be applied (name not Latest)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          'minor',
          SuggestionCategory.Updateable,
          '1.0.5',
          SuggestionTypes.release
        ),
        parsedDependency: new PackageDependency(
          { name: 'pkg3', version: '1.0.0', path: '' },
          descriptors
        ),
        packageSource: PackageSourceType.Registry,
        type: PackageVersionType.Version,
        order: 0
      },
      // Should NOT be applied (type not Release)
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.Latest,
          SuggestionCategory.Updateable,
          '2.0.0-beta.1',
          SuggestionTypes.prerelease
        ),
        parsedDependency: new PackageDependency(
          { name: 'pkg4', version: '1.0.0', path: '' },
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

    const testEvent = new OnUpdateDependenciesLatestClick(
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
    verify(this.mockState.increaseBusyState()).once();
    verify(this.mockState.decreaseBusyState()).once();
    verify(this.mockState.enableCodeLensReplace(false)).once();
    verify(this.mockState.enableCodeLensReplace(true)).once();
    verify(mockWorkspaceEdit.replace(anything(), anything(), anything())).once();
    verify(this.mockWorkspace.applyEdit(instance(mockWorkspaceEdit))).once();
  },

  "handles error state": async function (this: TestContext) {
    // setup
    const testFileName = 'package.json';
    const mockEditor = {
      document: {
        fileName: testFileName,
        uri: { fsPath: testFileName, scheme: 'file' } as any
      }
    } as TextEditor;

    const mockProvider = mock<ISuggestionProvider>();
    when(this.mockGetSuggestionProvider.execute(testFileName)).thenReturn(instance(mockProvider));
    when(this.mockGetSuggestions.execute(instance(mockProvider), '.', testFileName, false))
      .thenReject(new Error('test error'));

    const testEvent = new OnUpdateDependenciesLatestClick(
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
    verify(this.mockState.increaseBusyState()).once();
    verify(this.mockState.setErrorState()).once();
    verify(this.mockState.decreaseBusyState()).once();
    verify(this.mockWorkspace.applyEdit(anything())).never();
  },

  "uses correct project path in workspace mode": async function (this: TestContext) {
    // setup
    const testFileName = 'd:/project/package.json';
    const testProjectPath = 'd:/project';
    const mockEditor = {
      document: {
        fileName: testFileName,
        uri: { fsPath: testFileName, scheme: 'file' } as any,
        positionAt: (offset: number) => ({ line: 0, character: offset } as any)
      }
    } as TextEditor;

    const mockProvider = mock<ISuggestionProvider>();
    when(this.mockExtension.isWorkspaceMode).thenReturn(true);
    when(this.mockExtension.projectPath).thenReturn(testProjectPath);
    when(this.mockGetSuggestionProvider.execute(testFileName)).thenReturn(instance(mockProvider));
    when(this.mockGetSuggestions.execute(instance(mockProvider), testProjectPath, testFileName, false))
      .thenResolve([]);

    const testEvent = new OnUpdateDependenciesLatestClick(
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
    verify(this.mockGetSuggestions.execute(
      instance(mockProvider),
      testProjectPath,
      testFileName,
      false
    )).once();
  },

  "deduplicates ranges": async function (this: TestContext) {
    // setup
    const testFileName = 'package.json';
    const mockProvider = mock<ISuggestionProvider>();

    const descriptors = new PackageDescriptor([
      {
        type: 'version' as any,
        version: '1.0.0',
        versionRange: { start: 10, end: 15 },
        versionPrepend: '',
        versionAppend: ''
      }
    ]);

    const testSuggestions: Array<PackageResponse> = [
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.Latest,
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
      // Duplicate range
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.Latest,
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
        order: 1
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

    const testEvent = new OnUpdateDependenciesLatestClick(
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

  "uses provider.suggestionReplaceFn if available": async function (this: TestContext) {
    // setup
    const testFileName = 'package.json';
    const mockProvider = mock<ISuggestionProvider>();
    (instance(mockProvider) as any).suggestionReplaceFn = (update: any, version: string) => `custom-${version}`;

    const descriptors = new PackageDescriptor([
      {
        type: 'version' as any,
        version: '1.0.0',
        versionRange: { start: 10, end: 15 },
        versionPrepend: '',
        versionAppend: ''
      }
    ]);

    const testSuggestions: Array<PackageResponse> = [
      {
        providerName: 'test-provider',
        suggestion: createSuggestion(
          SuggestionStatusText.Latest,
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
      } as PackageResponse
    ];

    const mockEditor = {
      document: {
        fileName: testFileName,
        uri: { fsPath: testFileName, scheme: 'file' } as any,
        positionAt: (offset: number) => ({ line: 0, character: offset } as any)
      }
    } as TextEditor;

    const mockWorkspaceEdit = mock<WorkspaceEdit>();

    when(this.mockGetSuggestionProvider.execute(testFileName)).thenReturn(instance(mockProvider));
    when(this.mockGetSuggestions.execute(instance(mockProvider), '.', testFileName, false))
      .thenResolve(testSuggestions);
    when(this.mockConstruct.createWorkspaceEdit()).thenReturn(instance(mockWorkspaceEdit));
    when(this.mockConstruct.createRange(anything(), anything())).thenReturn({} as any);

    const testEvent = new OnUpdateDependenciesLatestClick(
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
    verify(mockWorkspaceEdit.replace(anything(), anything(), 'custom-1.1.0')).once();
    verify(this.mockWorkspace.applyEdit(instance(mockWorkspaceEdit))).once();
  },

};
