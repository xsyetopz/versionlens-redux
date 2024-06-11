import { IStorage } from '#domain/storage';
import { readFile } from 'node:fs/promises';
import { TextDecoder } from 'node:util';

export const CrLf = '\r\n';
export const Lf = '\n';

export class FileSystemStorage implements IStorage {

  async readFile(absFilePath: string): Promise<string> {
    const buffer = await readFile(absFilePath);
    // strip any BOMs from the buffer
    const decoder = new TextDecoder('utf-8', { ignoreBOM: false });
    // return the string
    return decoder.decode(buffer);
  }

  async readJsonFile<T>(absFilePath: string): Promise<T> {
    const jsonContent = await this.readFile(absFilePath);
    return JSON.parse(jsonContent);
  }

}