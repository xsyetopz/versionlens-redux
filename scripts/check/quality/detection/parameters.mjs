const LEADING_UNDERSCORES_PATTERN = /^_+/u;

function escapeRegExp(source) {
  return source.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function isReceiver(name) {
  return !name || name === "self" || name === "&self" || name === "&mut self";
}

function collectParameters(functions, stripComments) {
  const suppressedParameters = [];
  const unusedParameters = [];
  for (const fn of functions) {
    const bodyForUsage = stripComments(fn.body);
    for (const parameter of fn.parameters) {
      recordParameter({
        bodyForUsage,
        fn,
        parameter,
        suppressedParameters,
        unusedParameters,
      });
    }
  }
  return { suppressedParameters, unusedParameters };
}

function recordParameter({
  fn,
  parameter,
  bodyForUsage,
  suppressedParameters,
  unusedParameters,
}) {
  if (isReceiver(parameter.name)) {
    return;
  }
  const finding = {
    functionName: fn.name,
    line: fn.startLine,
    parameterName: parameter.name,
    path: fn.path,
  };
  if (parameter.name.startsWith("_") && parameter.name !== "_") {
    suppressedParameters.push(finding);
  }
  const usableName = parameter.name.replace(LEADING_UNDERSCORES_PATTERN, "");
  if (!usableName) {
    return;
  }
  const parameterUsagePattern = new RegExp(
    `\\b${escapeRegExp(usableName)}\\b`,
    "u",
  );
  if (!parameterUsagePattern.test(bodyForUsage)) {
    unusedParameters.push(finding);
  }
}

export { collectParameters };
