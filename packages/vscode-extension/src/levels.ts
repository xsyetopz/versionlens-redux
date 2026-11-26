/** The update levels, in menu order. */
export const LEVELS = ["latest", "major", "minor", "patch"] as const;

/** A client command that runs a server command at one level, for menus that pass no level. */
export interface LevelCommand {
	id: string;
	level: (typeof LEVELS)[number];
	server: "versionlens.updateAll" | "versionlens.updateWorkspace";
}

/** One command per level for the file update and the workspace update. */
export const LEVEL_COMMANDS: LevelCommand[] = LEVELS.flatMap((level) => [
	{
		id: `versionlens.updateAll.${level}`,
		level,
		server: "versionlens.updateAll"
	},
	{
		id: `versionlens.updateWorkspace.${level}`,
		level,
		server: "versionlens.updateWorkspace"
	}
]);
