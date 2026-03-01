import fs from 'node:fs';
import util from 'node:util';

export const CrLf = '\r\n';
export const Lf = '\n';

// setup fs/promises for node v20
const fsReadFile = util.promisify(fs.readFile);
const access = util.promisify(fs.access);

/**
 * Checks if a file exists at the specified absolute path.
 * @param absFilePath The absolute path to the file.
 * @returns A promise resolving to true if the file exists, otherwise false.
 */
export async function fileExists(absFilePath: string): Promise<boolean> {
  try {
    await access(absFilePath);
    return true;
  } catch (error: any) {
    return false;
  }
}

/**
 * Reads the content of a file as a UTF-8 string.
 * @param absFilePath The absolute path to the file.
 * @returns A promise resolving to the file content.
 */
export function readFile(absFilePath: string): Promise<string> {
  return fsReadFile(absFilePath, "utf8")
}