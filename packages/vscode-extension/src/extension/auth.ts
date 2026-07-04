import { authHeaders } from "./auth/headers.ts";
import { removeAuthHeader } from "./auth/remove.ts";
import { addAuthHeader, isAuthHeaderSuppressed } from "./auth/set.ts";

export { addAuthHeader, authHeaders, isAuthHeaderSuppressed, removeAuthHeader };
