import type { ILogger } from '#domain/logging';
import type { DependencyCache, PackageDependency } from '#domain/packages';
import type { IProviderConfig, ISuggestionProvider } from '#domain/providers';
import type { GetDependencyChanges } from '#domain/useCases';
import type { EditorConfig, IVsCodeWorkspace } from '#extension/vscode';
import { defaultExcludes, PackageFileWatcher } from '#extension/watcher';
import { test } from 'mocha-ui-esm';
import { anything, instance, mock, verify, when } from 'ts-mockito';
import type { FileSystemWatcher, Uri } from 'vscode';

type TestContext = {
  mockGetDependencyChanges: GetDependencyChanges
  mockProvider: ISuggestionProvider
  mockCache: DependencyCache
  mockProviderConfig: IProviderConfig;
  mockEditorConfig: EditorConfig
  mockWorkspace: IVsCodeWorkspace
  mockLogger: ILogger
  mockPackageFileWatcher: PackageFileWatcher
}

export const packageFileWatcherTests = {

  [test.title]: PackageFileWatcher.name,

  beforeEach: function (this: TestContext) {
    this.mockGetDependencyChanges = mock<GetDependencyChanges>();
    this.mockProvider = mock<ISuggestionProvider>();
    this.mockCache = mock<DependencyCache>();
    this.mockWorkspace = mock<IVsCodeWorkspace>();
    this.mockLogger = mock<ILogger>();
    this.mockProviderConfig = mock<IProviderConfig>();
    this.mockEditorConfig = mock<EditorConfig>();
    this.mockPackageFileWatcher = mock<PackageFileWatcher>();

    when(this.mockProvider.name).thenReturn("test provider");
    when(this.mockProviderConfig.filePatterns).thenReturn('**/package.json');
    when(this.mockProviderConfig.fileExcludePatterns).thenReturn(['**/node_modules/**']);
  },

  watchFolder: {
    "finds files using a provider file pattern": async function (this: TestContext) {
      // setup
      const testProvider = instance(this.mockProvider);
      const testConfig = instance(this.mockProviderConfig);
      const testUri: Uri = <any>{ fsPath: 'some-dir/package.json' };
      const testUserFileExcludes: Record<string, boolean> = { '**/exclude.files/**': true };
      const testExcludes = [
        ...defaultExcludes,
        ...Object.keys(testUserFileExcludes).filter(x => testUserFileExcludes[x]),
        ...testConfig.fileExcludePatterns!
      ];

      when(this.mockProvider.config).thenReturn(testConfig);
      when(this.mockEditorConfig.excludeFiles).thenReturn(testUserFileExcludes);

      when(
        this.mockWorkspace.findFiles(
          testConfig.filePatterns,
          `{${testExcludes.join(',')}}`
        )
      ).thenResolve([testUri] as any);

      const watcher = new PackageFileWatcher(
        instance(this.mockGetDependencyChanges),
        [testProvider],
        instance(this.mockCache),
        instance(this.mockEditorConfig),
        instance(this.mockWorkspace),
        instance(this.mockLogger)
      );

      // override dependent functions with mocks
      const stubWatcher = instance(this.mockPackageFileWatcher);
      watcher.onFileAdd = stubWatcher.onFileAdd;
      watcher.watch = stubWatcher.watch;

      // test
      await watcher.watchFolder();

      // verify
      verify(this.mockPackageFileWatcher.onFileAdd(testProvider, testUri)).once();
      verify(this.mockPackageFileWatcher.watch()).once();
    },
  },

  watchFile: {
    "finds files using a provider file pattern": async function (this: TestContext) {
      // setup
      const testProvider = instance(this.mockProvider);
      const testConfig = instance(this.mockProviderConfig);
      const testUri: Uri = <any>{ fsPath: 'some-dir/package.json' };

      when(this.mockProvider.config).thenReturn(testConfig);

      const watcher = new PackageFileWatcher(
        instance(this.mockGetDependencyChanges),
        [testProvider],
        instance(this.mockCache),
        instance(this.mockEditorConfig),
        instance(this.mockWorkspace),
        instance(this.mockLogger)
      );

      // override dependent functions with mocks
      const stubWatcher = instance(this.mockPackageFileWatcher);
      watcher.onFileAdd = stubWatcher.onFileAdd;
      watcher.watch = stubWatcher.watch;

      // test
      await watcher.watchFile(testUri);

      // verify
      verify(this.mockPackageFileWatcher.onFileAdd(testProvider, testUri)).once();
      verify(this.mockPackageFileWatcher.watch()).once();
    },
  },
  watch: {
    "watches files using a provider file pattern": async function (this: TestContext) {
      // setup
      const mockFileSystemWatcher = mock<FileSystemWatcher>();
      const testProvider = instance(this.mockProvider);
      const testConfig = instance(this.mockProviderConfig);

      when(this.mockProvider.config).thenReturn(testConfig);

      when(this.mockWorkspace.createFileSystemWatcher(testConfig.filePatterns))
        .thenReturn(instance(mockFileSystemWatcher))

      const watcher = new PackageFileWatcher(
        instance(this.mockGetDependencyChanges),
        [testProvider],
        instance(this.mockCache),
        instance(this.mockEditorConfig),
        instance(this.mockWorkspace),
        instance(this.mockLogger)
      );

      // test
      watcher.watch();

      // verify
      verify(
        this.mockLogger.debug(
          "created watcher for '{providerName}' with pattern '{filePatterns}'",
          testProvider.name,
          testConfig.filePatterns
        )
      ).once();

      verify(mockFileSystemWatcher.onDidCreate(anything())).once();
      verify(mockFileSystemWatcher.onDidDelete(anything())).once();
      verify(mockFileSystemWatcher.onDidChange(anything())).once();
    },
  },

  onFileChange: {

    "doesn't call changed listener when dependencies haven't changed": async function (this: TestContext) {
      // setup
      const stubWatcher = instance(this.mockPackageFileWatcher);
      const testProvider = instance(this.mockProvider);
      const testUri: Uri = <any>{ fsPath: 'some-dir/package.json' };
      const testDependencies: PackageDependency[] = [];

      when(this.mockGetDependencyChanges.execute(testProvider, testUri.fsPath))
        .thenResolve({
          parsedDependencies: testDependencies,
          hasChanged: false
        });

      const watcher = new PackageFileWatcher(
        instance(this.mockGetDependencyChanges),
        [],
        instance(this.mockCache),
        instance(this.mockEditorConfig),
        instance(this.mockWorkspace),
        instance(this.mockLogger)
      );

      // override dependent functions with mocks
      watcher.fire = stubWatcher.fire;

      // test
      await watcher.onFileChange(testProvider, testUri);

      // verify
      verify(this.mockLogger.trace("file changed '{uri}'", testUri.toString())).once();

      verify(
        this.mockGetDependencyChanges.execute(testProvider, testUri.fsPath),
      ).once()

      verify(
        this.mockCache.set(testProvider.name, testUri.fsPath, testDependencies)
      ).once();

      verify(
        this.mockPackageFileWatcher.fire(
          anything(),
          anything(),
          anything()
        )
      ).never();
    },

    "calls changed listener when dependencies have changed": async function (this: TestContext) {
      // setup
      const stubWatcher = instance(this.mockPackageFileWatcher);
      const testProvider = instance(this.mockProvider);
      const testUri: Uri = <any>{ fsPath: 'some-dir/package.json' };
      const testNewDependencies = [
        <PackageDependency>{
          package: {
            name: "test-package",
            version: "1.2.3",
            path: testUri.fsPath
          }
        }
      ];

      when(this.mockGetDependencyChanges.execute(testProvider, testUri.fsPath))
        .thenResolve({
          parsedDependencies: testNewDependencies,
          hasChanged: true
        });

      const watcher = new PackageFileWatcher(
        instance(this.mockGetDependencyChanges),
        [],
        instance(this.mockCache),
        instance(this.mockEditorConfig),
        instance(this.mockWorkspace),
        instance(this.mockLogger)
      );

      watcher.fire = stubWatcher.fire;

      // test
      await watcher.onFileChange(testProvider, testUri);

      // verify
      verify(
        this.mockLogger.trace("file changed '{uri}'", testUri.toString())
      ).once();

      verify(
        this.mockGetDependencyChanges.execute(testProvider, testUri.fsPath),
      ).once()

      verify(
        this.mockLogger.trace(
          "updating package dependency cache for '{packageFilePath}'",
          testUri.fsPath
        )
      ).once();

      verify(
        this.mockCache.set(
          testProvider.name,
          testUri.fsPath,
          testNewDependencies
        )
      ).once();

      verify(
        this.mockPackageFileWatcher.fire(
          testProvider,
          testUri.fsPath,
          testNewDependencies
        )
      ).once();
    }
  }
};