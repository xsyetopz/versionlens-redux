import { throwNotStringOrEmpty } from '@esm-test/guards';
import { commands } from 'vscode';
import { IContextState } from '../definitions';

/**
 * Manages a single piece of state that is synchronized with a VS Code context key.
 * @template T The type of the state value.
 */
export class ContextState<T> implements IContextState<T> {

  /**
   * Initializes a new instance of the ContextState class.
   * @param key The VS Code context key name.
   */
  constructor(private readonly key: string) {
    throwNotStringOrEmpty("key", key);
  }

  /** The internal value of the state. */
  private _value!: T;

  /**
   * Gets the current value of the state.
   */
  get value(): T {
    return this._value;
  }

  /**
   * Updates the state value and synchronizes it with VS Code.
   * @param newValue The new value to set.
   * @returns A promise resolving to the new value.
   */
  async change(newValue: T): Promise<T> {
    this._value = newValue;
    return await commands.executeCommand(
      'setContext',
      this.key,
      newValue
    );
  }

}