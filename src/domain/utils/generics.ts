/**
 * Represents an asynchronous function that returns a promise of type T.
 * @template T The return type of the promise.
 */
export type AsyncFunction<T> = (...args: Array<any>) => Promise<T>;

/** Constant used to check if a function is an AsyncFunction. */
export const AsyncFunction = async function () { }.constructor;

/**
 * Represents a dictionary with string keys and values of type T.
 * @template T The type of the values in the dictionary.
 */
export type KeyDictionary<T> = { [key: string]: T }

/**
 * Represents a dictionary with string keys and string values.
 */
export type KeyStringDictionary = KeyDictionary<string>

/**
 * Creates a dictionary of property names for a given type T.
 * @template T The type to extract property names from.
 */
export type PropertyNameDictionary<T> = { [Key in keyof T]: Key }

/**
 * Represents a type that can be null.
 * @template T The base type.
 */
export type Nullable<T> = T | null;