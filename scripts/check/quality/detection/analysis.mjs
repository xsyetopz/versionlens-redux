import { collectComplexTypes } from "./complex.mjs";
import { collectDuplicates } from "./duplicates.mjs";
import { extractFunctions as extractSourceFunctions } from "./extraction.mjs";
import { collectParameters } from "./parameters.mjs";
import { isTestPath } from "./paths.mjs";
import { collectQualifiedPaths } from "./qualified.mjs";
import { extractObjectShapes } from "./shapes.mjs";
import { matchingIndex, stripComments } from "./syntax.mjs";
import { collectWrappers } from "./wrappers.mjs";

const MAX_FIELDS = 10;
const MAX_PARAMETERS = 5;

function normalizedFiles(files) {
  return files.map((file) => ({
    ...file,
    source: file.source.replaceAll("\r\n", "\n"),
  }));
}

function oversizedFunctionsFrom(functions) {
  return functions
    .filter((fn) => fn.parameters.length > MAX_PARAMETERS)
    .map((fn) => ({
      language: fn.language,
      line: fn.startLine,
      name: fn.name,
      parameterCount: fn.parameters.length,
      path: fn.path,
    }));
}

function analyzeSources(files, options = {}) {
  const parsedFiles = normalizedFiles(files);
  const functions = parsedFiles.flatMap(extractSourceFunctions);
  const objectShapes = parsedFiles.flatMap(extractObjectShapes);
  const { suppressedParameters, unusedParameters } = collectParameters(
    functions,
    stripComments,
  );

  return {
    duplicateLogic: collectDuplicates(functions, options, {
      isTestPath,
      stripComments,
    }),
    overqualifiedPaths: collectQualifiedPaths(parsedFiles, options, {
      isTestPath,
      stripComments,
    }),
    oversizedFunctions: oversizedFunctionsFrom(functions),
    oversizedShapes: objectShapes.filter(
      (shape) => shape.fieldCount > MAX_FIELDS,
    ),
    passThroughWrappers: collectWrappers(functions, options, {
      isTestPath,
      matchingIndex,
      stripComments,
    }),
    repeatedComplexTypes: collectComplexTypes(functions, parsedFiles, options),
    suppressedParameters,
    unusedParameters,
  };
}

const extractFunctions = extractSourceFunctions;

export { analyzeSources, extractFunctions };
