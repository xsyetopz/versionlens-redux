import { PackageVersionType } from '#domain/packages';
import { NugetVersionSpec } from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';

export type DotNetVersionSpec = {
  type: PackageVersionType,
  rawVersion: string,
  resolvedVersion: string,
  spec: NugetVersionSpec,
};

export type DotNetSource = {
  enabled: boolean,
  machineWide: boolean,
  url: string,
  protocol: RegistryProtocols,
}