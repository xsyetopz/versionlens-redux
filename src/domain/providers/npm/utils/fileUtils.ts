import { KeyStringDictionary, fileExists, readFile } from '#domain/utils';
import dotenv from 'dotenv';
import { resolve } from 'node:path';

export async function resolveDotFilePath(
  dotFileName: string,
  cwds: Array<string>
): Promise<string> {
  for (const cwd of cwds) {
    const checkPath = resolve(cwd, dotFileName);
    const dotFileExists = await fileExists(checkPath);
    if (dotFileExists) return checkPath;
  }

  return '';
}

export async function getDotEnv(envPath: string): Promise<KeyStringDictionary> {
  // return the parsed env object
  return dotenv.parse(await readFile(envPath));
}