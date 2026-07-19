import { createExtensionState } from "../state.ts";
import { expect, it } from "./runtime.ts";

it("extension state defaults suggestion stats to hidden", (): void => {
  const state = createExtensionState();

  expect(state.flags.showSuggestionStats).toBe(false);
});
