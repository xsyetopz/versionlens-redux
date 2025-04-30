import type { DockerRepository } from '#domain/providers/docker';

export default {
  test: [
    {
      "name": "dev-22-bullseye",
      "digest": "sha256:dc236f2c6601b3efcac3516cfd8263eab4e93cc04f930bc973f337a502c3a7da",
      "createdDate": "2024-05-06T18:25:17.0000000+00:00"
    },
    {
      "name": "dev-bookworm",
      "digest": "sha256:9cf28df37d2e7f378183e423154f10a42e65620dfd9ac0f0b208b147b6c42f32",
      "createdDate": "2024-05-06T18:25:17.0000000+00:00"
    },
    {
      "name": "dev-bullseye",
      "digest": "sha256:dc236f2c6601b3efcac3516cfd8263eab4e93cc04f930bc973f337a502c3a7da",
      "createdDate": "2024-05-06T18:25:17.0000000+00:00"
    },
    {
      "name": "dev-buster",
      "digest": "sha256:0bbce09c286a6e6d47bb0f4f57ba819cef4a7a3e09c3deab3f3cdf23846c27e7",
      "createdDate": "2024-05-06T18:25:17.0000000+00:00"
    },
    {
      "name": "latest",
      "digest": "sha256:ee45bc4188a3804b597f707ceeaea98c59f118722d40094eaba3d71ca00fd3c8",
      "createdDate": "2024-05-06T18:25:17.0000000+00:00"
    }
  ],
  expected: <DockerRepository[]>[
    {
      "name": "dev-22-bullseye",
      "digest": "sha256:dc236f2c6601b3efcac3516cfd8263eab4e93cc04f930bc973f337a502c3a7da"
    },
    {
      "name": "dev-bookworm",
      "digest": "sha256:9cf28df37d2e7f378183e423154f10a42e65620dfd9ac0f0b208b147b6c42f32"
    },
    {
      "name": "dev-bullseye",
      "digest": "sha256:dc236f2c6601b3efcac3516cfd8263eab4e93cc04f930bc973f337a502c3a7da"
    },
    {
      "name": "dev-buster",
      "digest": "sha256:0bbce09c286a6e6d47bb0f4f57ba819cef4a7a3e09c3deab3f3cdf23846c27e7"
    },
    {
      "name": "latest",
      "digest": "sha256:ee45bc4188a3804b597f707ceeaea98c59f118722d40094eaba3d71ca00fd3c8"
    }
  ]
}