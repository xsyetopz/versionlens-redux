/**
 * The original extension's command ids and the commands that replaced them, so keybindings and
 * tasks written for VersionLens 1.x keep working.
 */
export const ALIASES: Readonly<Record<string, string>> = {
	"versionlens.editor.onAddUrlAuthentication": "versionlens.addAuthentication",
	"versionlens.editor.onCustomInstall": "versionlens.install",
	"versionlens.editor.onHidePrereleaseVersions": "versionlens.hidePrereleases",
	"versionlens.editor.onHideVersionLenses": "versionlens.hide",
	"versionlens.editor.onRemoveUrlAuthentication":
		"versionlens.removeAuthentication",
	"versionlens.editor.onShowError": "versionlens.showError",
	"versionlens.editor.onShowPrereleaseVersions": "versionlens.showPrereleases",
	"versionlens.editor.onShowVersionLenses": "versionlens.show",
	"versionlens.editor.onSortDependencies": "versionlens.sort",
	"versionlens.editor.onUpdateDependenciesLatest":
		"versionlens.updateAll.latest",
	"versionlens.editor.onUpdateDependenciesMajor": "versionlens.updateAll.major",
	"versionlens.editor.onUpdateDependenciesMinor": "versionlens.updateAll.minor",
	"versionlens.editor.onUpdateDependenciesPatch": "versionlens.updateAll.patch",
	"versionlens.icons.showingProgress": "versionlens.progress",
	"versionlens.suggestion.onClearCache": "versionlens.clearCache"
};
