import { readFile } from 'node:fs/promises';
import { TextDecoder } from 'node:util';

/**
 * Interface for a storage provider.
 */
export interface IStorage {
  /**
   * Reads the content of a file.
   * @param absFilePath The absolute path to the file.
   * @returns A promise resolving to the file content as a string.
   */
  readFile(absFilePath: string): Promise<string>;
}

/**
 * Implementation of IStorage that interacts with the physical file system.
 */
export class FileSystemStorage implements IStorage {

  /**
   * Reads the content of a file from the physical file system.
   * @param absFilePath The absolute path to the file.
   * @returns A promise resolving to the file content as a string.
   */
  async readFile(absFilePath: string): Promise<string> {
    const buffer = await readFile(absFilePath);
    // strip any BOMs from the buffer
    const decoder = new TextDecoder('utf-8', { ignoreBOM: false });
    // return the string
    return decoder.decode(buffer);
  }

}