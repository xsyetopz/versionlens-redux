import type { ILogger } from '#domain/logging';
import {
  createPackageResource,
  PackageDependency,
  PackageSourceType,
  PackageVersionType,
  SuggestionCategory,
  SuggestionTypes
} from '#domain/packages';
import { createPackageNameDesc, createPackageVersionDesc, createTextRange, PackageDescriptor } from '#domain/parsers';
import type { IContextState, ISuggestionCodeLens, IVersionLensState } from '#extension';
import { OnChooseBuildClick } from '#extension/events';
import { SuggestionInteractions } from '#extension/suggestions';
import type { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { deepEqual, equal } from 'assert';
import { test } from 'mocha-ui-esm';
import { anything, instance, mock, verify, when, deepEqual as deepEq } from 'ts-mockito';

type TestContext = {
  mockInterations: SuggestionInteractions
  mockConstruct: IVsCodeConstructFactory
  mockWorkspace: IVsCodeWorkspace
  mockState: IVersionLensState
  mockLogger: ILogger
  mockCodeLensReplace: IContextState<boolean>
  testEvent: OnChooseBuildClick
}

export const onChooseBuildClickTests = {

  [test.title]: OnChooseBuildClick.name,

  beforeEach: function (this: TestContext) {
    this.mockInterations = mock<SuggestionInteractions>()
    this.mockConstruct = mock<IVsCodeConstructFactory>()
    this.mockWorkspace = mock<IVsCodeWorkspace>()
    this.mockState = mock<IVersionLensState>()
    this.mockLogger = mock<ILogger>()
    this.mockCodeLensReplace = mock<IContextState<boolean>>()
    this.testEvent = new OnChooseBuildClick(
      instance(this.mockInterations),
      instance(this.mockConstruct),
      instance(this.mockWorkspace),
      instance(this.mockState),
      instance(this.mockLogger)
    );
  },

  "prevents replace when disabled": async function (this: TestContext) {
    // setup
    const testCodelens = mock<ISuggestionCodeLens>()
    when(this.mockCodeLensReplace.value).thenReturn(false)
    when(this.mockState.codeLensReplace).thenReturn(instance(this.mockCodeLensReplace))
    // test
    await this.testEvent.execute(instance(testCodelens));
    // verify
    verify(this.mockInterations.chooseBuild(anything(), anything(), anything())).never();
  },

  "prevents replace when user cancels interaction": async function (this: TestContext) {
    // setup
    const testBuildVersions = ['1.0.0', '1.0.0+amd', '1.0.0+arm', '1.0.0+intel']
    const testVersion = testBuildVersions.join(',')
    const testPackageResp = {
      suggestion: {
        name: 'latest',
        version: testVersion,
        type: SuggestionTypes.release,
        category: SuggestionCategory.Build
      },
      providerName: 'test-provider',
      packageSource: PackageSourceType.Registry,
      type: PackageVersionType.Version,
      parsedDependency: new PackageDependency(
        createPackageResource('test-name', testVersion, 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('testPackage1', createTextRange(0, 0)),
          createPackageVersionDesc(testVersion, createTextRange(1, 1)),
        ])
      ),
      order: 0,
    }
    const testCodelens = mock<ISuggestionCodeLens>()
    when(testCodelens.packageResponse).thenReturn(testPackageResp)
    when(this.mockState.codeLensReplace).thenReturn(instance(this.mockCodeLensReplace))
    // test
    await this.testEvent.execute(instance(testCodelens));
    // verify
    verify(
      this.mockInterations.chooseBuild(
        deepEq(testBuildVersions),
        testPackageResp.parsedDependency.package.name,
        testPackageResp.parsedDependency.package.version,
      )
    ).once();
    verify(this.mockState.enableCodeLensReplace(anything())).never();
  },

  "applies edits to '$1' versions": async function (this: TestContext) {
    // setup
    const expectedVersion = '1.0.0+amd'
    const testBuildVersions = ['1.0.0', '1.0.0+amd', '1.0.0+arm', '1.0.0+intel']
    const testSuggestVersions = testBuildVersions.join(',')
    const testPickedVersion = '1.0.0+amd'
    const testPkgVersion = '1.0.0'
    const testDocumentUrl = 'test-url'
    const testReplaceRange = {}
    const mockCodeLens = mock<ISuggestionCodeLens>()
    when(mockCodeLens.documentUrl).thenReturn(testDocumentUrl as any)
    when(mockCodeLens.replaceRange).thenReturn(testReplaceRange as any)
    when(mockCodeLens.packageResponse).thenReturn({
      suggestion: {
        name: 'latest',
        version: testSuggestVersions,
        type: SuggestionTypes.release,
        category: SuggestionCategory.Latest
      },
      providerName: 'test-provider',
      packageSource: PackageSourceType.Registry,
      type: PackageVersionType.Version,
      parsedDependency: new PackageDependency(
        createPackageResource('test-name', testPkgVersion, 'test/path'),
        new PackageDescriptor([
          createPackageNameDesc('testPackage1', createTextRange(0, 0)),
          createPackageVersionDesc(testPkgVersion, createTextRange(1, 1)),
        ])
      ),
      order: 0,
    })
    when(this.mockCodeLensReplace.value).thenReturn(true)
    when(this.mockState.codeLensReplace).thenReturn(instance(this.mockCodeLensReplace))
    when(
      this.mockInterations.chooseBuild(
        deepEq(testBuildVersions),
        'test-name',
        testPkgVersion
      )
    ).thenResolve(testPickedVersion)
    const testEdit = {
      replace: function (actualUrl: any, actualRange: any, actualVersion: string) {
        equal(actualUrl, testDocumentUrl)
        deepEqual(actualRange, testReplaceRange)
        equal(actualVersion, expectedVersion)
      }
    }
    when(this.mockConstruct.createWorkspaceEdit()).thenReturn(testEdit as any)
    when(mockCodeLens.replaceVersionFn(anything(), expectedVersion)).thenReturn(expectedVersion)

    // test
    await this.testEvent.execute(instance(mockCodeLens));

    // verify
    verify(this.mockState.enableCodeLensReplace(false)).once();
    verify(mockCodeLens.replaceVersionFn(anything(), expectedVersion)).once()
    verify(this.mockConstruct.createWorkspaceEdit()).once();
    verify(this.mockWorkspace.applyEdit(testEdit as any)).once();
  }

};