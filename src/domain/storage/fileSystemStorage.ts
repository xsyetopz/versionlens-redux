import { readFile } from 'node:fs/promises';
import { TextDecoder } from 'node:util';

export interface IStorage {
  readFile(absFilePath: string): Promise<string>;
}

export class FileSystemStorage implements IStorage {

  async readFile(absFilePath: string): Promise<string> {
    const buffer = await readFile(absFilePath);
    // strip any BOMs from the buffer
    const decoder = new TextDecoder('utf-8', { ignoreBOM: false });
    // return the string
    return decoder.decode(buffer);
  }

}