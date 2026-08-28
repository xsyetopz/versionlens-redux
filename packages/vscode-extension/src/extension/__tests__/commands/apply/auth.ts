import { createAuthContext } from "../../support.ts";
import {
  secretValues,
  storedSecrets,
  updatedConfig,
  workspaceValues,
} from "./support.ts";

function authContext(): ReturnType<typeof createAuthContext> {
  return createAuthContext({
    secretValues,
    storedSecrets,
    updatedSettings: updatedConfig,
    workspaceValues,
  });
}

export { authContext };
