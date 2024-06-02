import { OnClearCache } from "./commands/onClearCache";
import { OnFileLinkClick } from "./commands/onFileLinkClick";
import { OnUpdateDependencyClick } from "./commands/onUpdateDependencyClick";
import { OnErrorClick } from "./editorTitleBar/onErrorClick";
import { OnTogglePrereleases } from "./editorTitleBar/onTogglePrereleases";
import { OnToggleReleases } from "./editorTitleBar/onToggleReleases";
import { OnPreSaveChanges } from "./install/onPreSaveChanges";
import { OnSaveChanges } from "./install/onSaveChanges";
import { OnProviderEditorActivated } from "./provider/onProviderEditorActivated";
import { OnProviderTextDocumentChange } from "./provider/onProviderTextDocumentChange";
import { OnProviderTextDocumentClose } from "./provider/onProviderTextDocumentClose";
import { OnActiveTextEditorChange } from "./vscode/onActiveTextEditorChange";
import { OnTextDocumentChange } from "./vscode/onTextDocumentChange";
import { OnTextDocumentClose } from "./vscode/onTextDocumentClose";
import { OnTextDocumentSave } from "./vscode/onTextDocumentSave";
import { OnPackageDependenciesChanged } from "./watcher/onPackageDependenciesChanged";

// event di dependencies
export interface IEventServices {
  // command events
  onClearCache: OnClearCache;
  onFileLinkClick: OnFileLinkClick;
  onUpdateDependencyClick: OnUpdateDependencyClick;

  // editorTitleBar events
  onToggleReleases: OnToggleReleases;
  onTogglePrereleases: OnTogglePrereleases;
  onErrorClick: OnErrorClick;

  // install events
  onPreSaveChanges: OnPreSaveChanges;
  onSaveChanges: OnSaveChanges;

  // provider events
  onProviderEditorActivated: OnProviderEditorActivated;
  onProviderTextDocumentChange: OnProviderTextDocumentChange;
  onProviderTextDocumentClose: OnProviderTextDocumentClose;

  // vscode events
  onActiveTextEditorChange: OnActiveTextEditorChange;
  onTextDocumentChange: OnTextDocumentChange;
  onTextDocumentClose: OnTextDocumentClose;
  onTextDocumentSave: OnTextDocumentSave;

  // watcher events
  onPackageDependenciesChanged: OnPackageDependenciesChanged
}