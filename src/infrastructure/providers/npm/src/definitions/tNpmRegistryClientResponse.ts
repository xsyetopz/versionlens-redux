import { TClientResponse } from '#domain/clients';
import { KeyDictionary } from '#domain/utils';

export type TNpmRegistryData = {

  name: string;

  versions: KeyDictionary<any>;

  "dist-tags": KeyDictionary<string>;

}

export type TNpmRegistryClientResponse = TClientResponse<number, TNpmRegistryData>
