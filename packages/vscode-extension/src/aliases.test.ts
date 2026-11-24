import { expect, test } from "bun:test";

import manifest from "../package.json";
import { ALIASES } from "./aliases.ts";

test("every original command id forwards to a contributed command", () => {
	// Arrange
	const contributed = new Set(
		manifest.contributes.commands.map(({ command }) => command)
	);

	// Act
	const missing = Object.values(ALIASES).filter((id) => !contributed.has(id));

	// Assert
	expect(Object.keys(ALIASES)).toHaveLength(15);
	expect(missing).toEqual([]);
});
