import type { KeyDictionary } from '#domain/utils';
import type { UrlAuthenticationData } from '#extension/authorization';
import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';
import type { Memento } from 'vscode';

/**
 * Handles persistence of URL authentication metadata using VS Code Memento.
 */
export class UrlAuthenticationStore {

  /**
   * Initializes a new instance of the UrlAuthenticationStore class.
   * @param storeKey The key used to store the collection in Memento.
   * @param store The VS Code Memento storage (e.g., workspaceState).
   */
  constructor(readonly storeKey: string, readonly store: Memento) {
    throwNotStringOrEmpty('storeKey', storeKey);
    throwUndefinedOrNull('store', store);
  }

  /**
   * Gets authentication data for a specific URL.
   * @param url The URL to retrieve data for.
   * @returns The authentication data, or undefined if not found.
   */
  get(url: string): UrlAuthenticationData {
    return this.getCollection()[url];
  }

  /**
   * Gets all stored authentication data.
   * @returns An array of all URL authentication data entries.
   */
  getAll(): UrlAuthenticationData[] {
    const results = [];
    const all = this.getCollection();
    for (const key in all) {
      results.push(all[key]);
    }

    return results;
  }

  /**
   * Removes authentication data for a specific URL.
   * @param url The URL to remove data for.
   */
  async remove(url: string): Promise<void> {
    const map = this.getCollection();
    delete map[url];
    await this.store.update(this.storeKey, map);
  }

  /**
   * Updates or adds authentication data for a specific URL.
   * @param url The URL to update data for.
   * @param value The new authentication data.
   */
  async update(url: string, value: UrlAuthenticationData): Promise<void> {
    const map = this.getCollection();
    map[url] = value;
    await this.store.update(this.storeKey, map);
  }

  /**
   * Clears all stored authentication data.
   */
  async clear() {
    await this.store.update(this.storeKey, {});
  }

  /**
   * Internal method to retrieve the raw collection from storage.
   */
  private getCollection(): KeyDictionary<UrlAuthenticationData> {
    return this.store.get(this.storeKey, {});
  }

}