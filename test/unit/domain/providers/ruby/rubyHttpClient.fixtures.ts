import { type JsonClientResponse, ClientResponseSource } from '#domain/clients';
import type { RubyHttpClientResponse } from '#domain/providers/ruby';

export const rubyHttpClientFixtures = {
  success: {
    data: [
      { number: '1.0.0' },
      { number: '1.1.0' },
      { number: '2.0.0-alpha' }
    ],
    status: 200,
    source: ClientResponseSource.remote
  } as JsonClientResponse<any[]>,
  notFound: {
    data: 'Not Found',
    status: 404,
    source: ClientResponseSource.remote
  } as JsonClientResponse<string>
};

export const rubyHttpClientResultFixtures = {
  success: {
    data: ['1.0.0', '1.1.0', '2.0.0-alpha'],
    status: 200,
    source: ClientResponseSource.remote
  } as RubyHttpClientResponse
};
