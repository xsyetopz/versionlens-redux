export type AsyncFunction<T> = (...args: Array<any>) => Promise<T>;

export const AsyncFunction = async function () { }.constructor;

export type KeyDictionary<T> = { [key: string]: T }

export type KeyStringDictionary = KeyDictionary<string>

export type PropertyNameDictionary<T> = { [Key in keyof T]: Key }

export type Undefinable<T> = T | undefined;

export type Nullable<T> = T | null;