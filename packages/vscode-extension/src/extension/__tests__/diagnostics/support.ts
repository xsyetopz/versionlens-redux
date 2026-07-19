import { mock, mockVscodeHost } from "../runtime.ts";
import { createVscodeMock } from "./mock.ts";
import { createNativeMock } from "./native-mock.ts";

mockVscodeHost(createVscodeMock);
mock.module("../../native/module.ts", createNativeMock);
