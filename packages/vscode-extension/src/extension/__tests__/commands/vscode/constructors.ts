interface MockEventEmitter {
  dispose: () => void;
  event: (listener: () => void) => { dispose: () => void };
  fire: () => void;
}

type MockEventEmitterConstructor = (this: MockEventEmitter) => void;

type RelativePatternConstructor = (
  this: { base: unknown; pattern: string },
  base: unknown,
  pattern: string,
) => void;

type SimpleConstructor = (this: Record<string, never>) => void;

interface MockWorkspaceEdit {
  edits: unknown[];
  replace: (uri: unknown, range: unknown, newText: string) => void;
}

type WorkspaceEditConstructor = new () => MockWorkspaceEdit;

function createCodeLensConstructor(): SimpleConstructor {
  function mockCodeLens(this: Record<string, never>): void {
    Object.assign(this, {});
  }
  return mockCodeLens;
}

function createEventEmitterConstructor(
  onFire: () => void,
): MockEventEmitterConstructor {
  function mockEventEmitterConstructor(this: MockEventEmitter): void {
    const listeners: (() => void)[] = [];
    this.dispose = (): void => {
      listeners.length = 0;
    };
    this.event = (listener: () => void): { dispose: () => void } => {
      listeners.push(listener);
      return {
        dispose(): void {
          const index = listeners.indexOf(listener);
          if (index >= 0) {
            listeners.splice(index, 1);
          }
        },
      };
    };
    this.fire = (): void => {
      onFire();
      for (const listener of [...listeners]) {
        listener();
      }
    };
  }
  return mockEventEmitterConstructor;
}

function createRelativePatternConstructor(): RelativePatternConstructor {
  function mockRelativePattern(
    this: { base: unknown; pattern: string },
    base: unknown,
    pattern: string,
  ): void {
    this.base = base;
    this.pattern = pattern;
  }
  return mockRelativePattern;
}

function createWorkspaceEditConstructor(): WorkspaceEditConstructor {
  return class MockWorkspaceEditConstructor implements MockWorkspaceEdit {
    edits: unknown[] = [];

    replace(uri: unknown, range: unknown, newText: string): void {
      this.edits.push({ newText, range, uri });
    }
  };
}

export {
  createCodeLensConstructor,
  createEventEmitterConstructor,
  createRelativePatternConstructor,
  createWorkspaceEditConstructor,
};
