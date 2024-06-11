import { ClientResponseSource } from '#domain/clients';

export type TPackageClientResponseStatus = {

  source: ClientResponseSource;

  status: number;

  rejected?: boolean;

};
