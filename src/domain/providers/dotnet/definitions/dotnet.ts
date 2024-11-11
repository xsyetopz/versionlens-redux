import { UrlUtils } from '#domain/clients';
import { PackageVersionType } from '#domain/packages';
import { NugetVersionSpec } from '#domain/providers/dotnet';

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
  protocol: UrlUtils.RegistryProtocols,
}