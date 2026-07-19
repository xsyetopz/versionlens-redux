import {
  registerCommonWrapperCase,
  registerCrossFileAliasCase,
  registerCrossFileConcreteTypeCase,
  registerFormatCaptureCase,
  registerLocalAliasCase,
  registerLocalConcreteTypeCase,
  registerMultilineParameterCase,
  registerQualityShapeCase,
  registerStringCommentCase,
} from "./analysis/type-cases.mjs";
import {
  registerQualityShapeCase0,
  registerQualityShapeCase1,
  registerQualityShapeCase2,
  registerQualityShapeCase3,
  registerQualityShapeCase4,
  registerQualityShapeCase5,
  registerQualityShapeCase6,
  registerQualityShapeCase7,
  registerQualityShapeCase8,
} from "./analysis/usage.mjs";

const { describe, expect, it } = Bun.jest(import.meta.path);

describe("check-code-quality", () => {
  for (const registerCase of [
    registerQualityShapeCase,
    registerMultilineParameterCase,
    registerLocalAliasCase,
    registerCrossFileAliasCase,
    registerLocalConcreteTypeCase,
    registerCrossFileConcreteTypeCase,
    registerCommonWrapperCase,
    registerFormatCaptureCase,
    registerStringCommentCase,
    registerQualityShapeCase0,
    registerQualityShapeCase1,
    registerQualityShapeCase2,
    registerQualityShapeCase3,
    registerQualityShapeCase4,
    registerQualityShapeCase5,
    registerQualityShapeCase6,
    registerQualityShapeCase7,
    registerQualityShapeCase8,
  ]) {
    registerCase({ it, expect });
  }
});
