import { expect, test } from "bun:test";

import { createExtensionState } from "./state.ts";

test("extension state defaults suggestion stats to hidden", () => {
	const state = createExtensionState();

	expect(state.flags.showSuggestionStats).toBe(false);
});
