import fs from 'node:fs';
import util from 'node:util';

export const CrLf = '\r\n';
export const Lf = '\n';

// setup fs/promises for node v20
const fsReadFile = util.promisify(fs.readFile);
const access = util.promisify(fs.access);

export async function fileExists(absFilePath: string): Promise<boolean> {
  try {
    await access(absFilePath);
    return true;
  } catch (error: any) {
    return false;
  }
}

export function readFile(absFilePath: string): Promise<string> {
  return fsReadFile(absFilePath, "utf8")
}

export async function readJsonFile<T>(absFilePath: string): Promise<T> {
  const jsonContent = await readFile(absFilePath)
  return JSON.parse(jsonContent);
}