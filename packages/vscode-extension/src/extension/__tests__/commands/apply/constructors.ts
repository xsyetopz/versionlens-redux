interface WorkspaceEditMock {
  edits: unknown[];
  replace: (uri: unknown, range: unknown, text: string) => void;
}

interface CodeLensMock {
  range: unknown;
}

interface RelativePatternMock {
  base: unknown;
  pattern: string;
}

function codeLens(this: CodeLensMock, lensRange: unknown): void {
  this.range = lensRange;
}

function relativePattern(
  this: RelativePatternMock,
  base: unknown,
  pattern: string,
): void {
  this.base = base;
  this.pattern = pattern;
}

function workspaceEdit(this: WorkspaceEditMock): void {
  this.edits = [];
  this.replace = (
    documentUri: unknown,
    editRange: unknown,
    newText: string,
  ): void => {
    this.edits.push({ newText, range: editRange, uri: documentUri });
  };
}

function range(): object {
  return {};
}

export { codeLens, range, relativePattern, workspaceEdit };
