import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createProjectVersionTypeDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {

  parsesItemGroupPackages: {

    test: `
      <Project>
        <Sdk Name="Microsoft.Build.CentralPackageVersions" Version="2.1.3" />
        <ItemGroup>
            <PackageReference Include="Microsoft.Extensions.DependencyInjection.Abstractions" Version="2.0.0" />
            <PackageReference Include="Microsoft.Extensions.Logging.Abstractions" Version="2.0.1" />
            <PackageVersion Include="System.Text.Json" Version="4.7.2" />
            <PackageVersion Include="Microsoft.Extensions.Options" VersionOverride="1.2.3" />
            <DotNetCliToolReference Include="Microsoft.EntityFrameworkCore.Tools" Version="6.0.7" />
            <GlobalPackageReference Include="Microsoft.Azure.ServiceBus" Version="(3.0,)" />
            <PackageVersion Update="AngularJS.Core" Version="1.0.*" />
            <PackageReference Include="NoVersionAttribute" />
            <PackageReference Include="ChildVersionNoAttribute">
              <!-- should ignore -->
              <Version></Version>
            </PackageReference>
        </ItemGroup>
      </Project>
    `,

    expected: [
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Build.CentralPackageVersions",
          createTextRange(25, 25)
        ),
        createPackageVersionDesc("2.1.3", createTextRange(85, 90))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Azure.ServiceBus",
          createTextRange(610, 610)
        ),
        createPackageVersionDesc("(3.0,)", createTextRange(680, 686))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Extensions.DependencyInjection.Abstractions",
          createTextRange(127, 127)
        ),
        createPackageVersionDesc("2.0.0", createTextRange(218, 223))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Extensions.Logging.Abstractions",
          createTextRange(240, 240)
        ),
        createPackageVersionDesc("2.0.1", createTextRange(319, 324))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "NoVersionAttribute",
          createTextRange(774, 774)
        ),
        createPackageVersionDesc("*", createTextRange(821, 821), 'Version="', '" ')
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "ChildVersionNoAttribute",
          createTextRange(836, 836)
        ),
        createPackageVersionDesc("*", createTextRange(887, 887), ' Version="', '"')
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "System.Text.Json",
          createTextRange(341, 341)
        ),
        createPackageVersionDesc("4.7.2", createTextRange(393, 398))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Extensions.Options",
          createTextRange(415, 415)
        ),
        createPackageVersionDesc("1.2.3", createTextRange(487, 492))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "AngularJS.Core",
          createTextRange(703, 703)
        ),
        createPackageVersionDesc("1.0.*", createTextRange(752, 757))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.EntityFrameworkCore.Tools",
          createTextRange(509, 509)
        ),
        createPackageVersionDesc("6.0.7", createTextRange(588, 593))
      ]),
    ]
  },

  parsesPropertyGroupVersions: {
    test: `
      <Project>
        <PropertyGroup>
            <AssemblyVersion>1.2.3</AssemblyVersion>
            <Version></Version>
        </PropertyGroup>
      </Project>
    `,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("AssemblyVersion", createTextRange(53, 53)),
        createPackageVersionDesc("1.2.3", createTextRange(70, 75)),
        createProjectVersionTypeDesc()
      ]),
      new PackageDescriptor([
        createPackageNameDesc("Version", createTextRange(106, 106)),
        createPackageVersionDesc("", createTextRange(115, 115)),
        createProjectVersionTypeDesc()
      ]),
    ]
  }
}