//! Infrastructure and environment manifests: Terraform modules, Terragrunt, Packer, helmfile,
//! Argo CD, Flux, Vagrant, Snapcraft, Flatpak, proto, aqua, Devbox, Spack, Scoop, winget, Pulumi, Terraform locks, GitLab CI, and `CircleCI`.

use super::expect;
use crate::catalog::file_type_of;

#[test]
fn reads_terraform_git_modules() {
    let text = "module \"vpc\" {\n  source = \"git::https://github.com/acme/modules.git//vpc?ref=v1.2.0\"\n}\nmodule \"dns\" {\n  source = \"github.com/hashicorp/example?ref=main\"\n}\n";
    expect(
        "infra/main.tf",
        text,
        &[("acme/modules", "v1.2.0", "github")],
    );
}

#[test]
fn reads_terragrunt_sources() {
    let text = "terraform {\n  source = \"tfr:///terraform-aws-modules/iam/aws//modules/iam-policy?version=4.3.0\"\n}\n# source = \"tfr://registry.opentofu.org/terraform-aws-modules/vpc/aws?version=3.3.0\"\n# source = \"git::https://github.com/acme/modules.git//vpc?ref=v1.0.0\"\n# source = \"../modules/networking/vpc\"\n";
    expect(
        "live/prod/terragrunt.hcl",
        text,
        &[
            ("terraform-aws-modules/iam/aws", "4.3.0", "terraform"),
            ("terraform-aws-modules/vpc/aws", "3.3.0", "terraform"),
            ("acme/modules", "v1.0.0", "github"),
        ],
    );
}

#[test]
fn reads_packer_required_plugins() {
    let text = "packer {\n  required_plugins {\n    happycloud = {\n      version = \">= 2.7.0\"\n      source = \"github.com/hashicorp/happycloud\"\n    }\n    amazon = {\n      source  = \"github.com/hashicorp/amazon\"\n      version = \"~> 1\"\n    }\n  }\n}\n";
    expect(
        "build.pkr.hcl",
        text,
        &[
            ("hashicorp/happycloud", ">= 2.7.0", "packer"),
            ("hashicorp/amazon", "~> 1", "packer"),
        ],
    );
}

#[test]
fn reads_helmfile_releases() {
    let text = "repositories:\n  - name: prometheus-community\n    url: https://prometheus-community.github.io/helm-charts\nreleases:\n  - name: prom\n    chart: prometheus-community/prometheus\n    version: 25.0.0\n    values:\n      - server:\n          persistentVolume: {}\n  - name: grafana\n    version: \"7.3.0\"\n    namespace: monitoring\n    chart: grafana/grafana\n  - name: local\n    chart: ./charts/local\n    version: 0.1.0\n  - name: unpinned\n    chart: jetstack/cert-manager\n";
    expect(
        "helmfile.yaml",
        text,
        &[
            ("prometheus-community/prometheus", "25.0.0", "artifacthub"),
            ("grafana/grafana", "7.3.0", "artifacthub"),
        ],
    );
}

#[test]
fn reads_argo_cd_and_flux_sources() {
    let argo = "apiVersion: argoproj.io/v1alpha1\nkind: Application\nspec:\n  source:\n    chart: sealed-secrets\n    repoURL: https://bitnami.github.io/sealed-secrets\n    targetRevision: 1.16.1\n    helm:\n      releaseName: sealed-secrets\n---\nkind: Application\nspec:\n  sources:\n    - repoURL: https://charts.example.com\n      chart: app\n      targetRevision: \"2.0.0\"\n    - repoURL: https://github.com/argoproj/argocd-example-apps.git\n      targetRevision: HEAD\n      path: guestbook\n";
    expect(
        "argocd/apps/secrets.yaml",
        argo,
        &[
            ("sealed-secrets", "1.16.1", "helm"),
            ("app", "2.0.0", "helm"),
        ],
    );
    let flux = "apiVersion: source.toolkit.fluxcd.io/v1\nkind: OCIRepository\nspec:\n  interval: 5m0s\n  url: oci://ghcr.io/stefanprodan/manifests/podinfo\n  ref:\n    tag: 6.1.6\n---\nkind: GitRepository\nspec:\n  url: https://github.com/stefanprodan/podinfo\n  ref:\n    tag: v6.6.0\n---\nkind: GitRepository\nspec:\n  url: https://github.com/stefanprodan/podinfo\n  ref:\n    branch: master\n";
    expect(
        "clusters/prod/sources.yaml",
        flux,
        &[
            ("ghcr.io/stefanprodan/manifests/podinfo", "6.1.6", "oci"),
            ("stefanprodan/podinfo", "v6.6.0", "github"),
        ],
    );
}

#[test]
fn reads_vagrant_boxes() {
    let text = "Vagrant.configure(\"2\") do |config|\n  config.vm.box = \"bento/ubuntu-22.04\"\n  config.vm.box_version = \"202502.21.0\"\nend\n";
    expect(
        "Vagrantfile",
        text,
        &[("bento/ubuntu-22.04", "202502.21.0", "vagrant")],
    );
}

#[test]
fn reads_snapcraft_and_flatpak_git_tags() {
    let snap = "parts:\n  app:\n    plugin: go\n    source: https://github.com/canonical/multipass.git\n    source-tag: v1.14.0\n  other:\n    source-tag: 2.0.1\n    source-type: git\n    source: https://github.com/o/other\n  local:\n    source: .\n";
    expect(
        "snap/snapcraft.yaml",
        snap,
        &[
            ("canonical/multipass", "v1.14.0", "github"),
            ("o/other", "2.0.1", "github"),
        ],
    );
    let json = "{\n    \"app-id\": \"com.github.tchx84.Flatseal\",\n    \"runtime-version\": \"47\",\n    \"modules\": [{\n        \"name\": \"flatseal\",\n        \"sources\": [{\n            \"type\": \"git\",\n            \"url\": \"https://github.com/tchx84/Flatseal.git\",\n            \"tag\": \"v2.4.1\",\n            \"commit\": \"3516a94d\"\n        }]\n    }]\n}\n";
    expect(
        "/work/com.github.tchx84.Flatseal.json",
        json,
        &[("tchx84/Flatseal", "v2.4.1", "github")],
    );
    let yaml = "id: org.example.App\nruntime-version: '47'\nmodules:\n  - name: lib\n    sources:\n      - type: git\n        url: https://github.com/o/lib.git\n        tag: 1.2.3\n";
    expect(
        "/work/org.example.App.yml",
        yaml,
        &[("o/lib", "1.2.3", "github")],
    );
    assert!(file_type_of("/work/com.example.settings.json", r#"{"theme": "dark"}"#).is_none());
}

#[test]
fn reads_tool_manager_pins() {
    let proto = "node = \"16.16.0\"\nnpm = \"9\"\ngo = \"~1.20\"\nrust = \"stable\"\npython = \"3.12.1\"\n\"npm:typescript\" = \"5\"\n\n[tools.node.aliases]\nwork = \"22\"\n";
    expect(
        ".prototools",
        proto,
        &[
            ("node", "16.16.0", "npm"),
            ("npm", "9", "npm"),
            ("go", "~1.20", "golang"),
            ("python", "3.12.1", "python"),
            ("typescript", "5", "npm"),
        ],
    );
    let aqua = "registries:\n  - type: standard\n    ref: v4.593.0\npackages:\n  - name: cli/cli@v2.2.0\n  - name: kubernetes/kubernetes/kubectl@v1.30.0\n  - name: suzuki-shunsuke/tfcmt\n    version: v4.0.0\n";
    expect(
        "aqua.yaml",
        aqua,
        &[
            ("cli/cli", "v2.2.0", "github"),
            ("kubernetes/kubernetes", "v1.30.0", "github"),
            ("suzuki-shunsuke/tfcmt", "v4.0.0", "github"),
        ],
    );
}

#[test]
fn reads_devbox_lists_and_maps() {
    let list = r#"{"packages": ["go@1.21", "nodejs@latest", "ripgrep"]}"#;
    expect("devbox.json", list, &[("go", "1.21", "nixhub")]);
    let map = r#"{"packages": {"fd": "10.1.0", "nodejs-slim": {"version": "22.4", "disable_plugin": true}}}"#;
    expect(
        "devbox.json",
        map,
        &[
            ("fd", "10.1.0", "nixhub"),
            ("nodejs-slim", "22.4", "nixhub"),
        ],
    );
}

#[test]
fn reads_spack_environment_specs() {
    let text = "spack:\n  specs:\n  - hdf5+mpi\n  - zlib@1.2.8\n  - gcc@=15.2 +binutils\n  - openmpi@4.1:4.2\n  concretizer:\n    unify: true\n";
    expect(
        "spack.yaml",
        text,
        &[
            ("zlib", "1.2.8", "spack"),
            ("gcc", "15.2", "spack"),
            ("openmpi", "4.1:4.2", "spack"),
        ],
    );
}

#[test]
fn reads_scoop_release_urls() {
    // ScoopInstaller/Main's bucket/gh.json; `autoupdate` URLs hold `$version`, not a tag.
    let text = "{\n    \"version\": \"2.105.0\",\n    \"architecture\": {\n        \"64bit\": {\n            \"url\": \"https://github.com/cli/cli/releases/download/v2.105.0/gh_2.105.0_windows_amd64.zip\",\n            \"hash\": \"8813a7\"\n        }\n    },\n    \"autoupdate\": {\n        \"architecture\": {\n            \"64bit\": {\n                \"url\": \"https://github.com/cli/cli/releases/download/v$version/gh_$version_windows_amd64.zip\"\n            }\n        }\n    }\n}\n";
    expect("bucket/gh.json", text, &[("cli/cli", "v2.105.0", "github")]);
}

#[test]
fn reads_winget_installer_urls() {
    // microsoft/winget-pkgs, BurntSushi.ripgrep.MSVC.installer.yaml at 16.0.0.
    let text = "PackageIdentifier: BurntSushi.ripgrep.MSVC\nPackageVersion: 16.0.0\nInstallers:\n- Architecture: x64\n  InstallerUrl: https://github.com/BurntSushi/ripgrep/releases/download/16.0.0/ripgrep-16.0.0-x86_64-pc-windows-msvc.zip\n  InstallerSha256: F418AE83\nManifestType: installer\n";
    expect(
        "manifests/b/BurntSushi/ripgrep/MSVC/16.0.0/BurntSushi.ripgrep.MSVC.installer.yaml",
        text,
        &[("BurntSushi/ripgrep", "16.0.0", "github")],
    );
}

#[test]
fn reads_flux_helm_release_charts() {
    // The Flux HelmRelease spec's HelmRepository example, then the version before the source.
    let text = "apiVersion: source.toolkit.fluxcd.io/v1\nkind: HelmRepository\nmetadata:\n  name: podinfo\nspec:\n  url: https://stefanprodan.github.io/podinfo\n---\napiVersion: helm.toolkit.fluxcd.io/v2\nkind: HelmRelease\nmetadata:\n  name: podinfo\nspec:\n  chart:\n    spec:\n      chart: podinfo\n      sourceRef:\n        kind: HelmRepository\n        name: podinfo\n      version: \"6.x\"\n---\napiVersion: helm.toolkit.fluxcd.io/v2\nkind: HelmRelease\nmetadata:\n  name: cert-manager\nspec:\n  chart:\n    spec:\n      chart: cert-manager\n      version: \"v1.16.2\"\n      sourceRef:\n        kind: HelmRepository\n        name: cert-manager\n        namespace: flux-system\n";
    expect(
        "clusters/production/podinfo.yaml",
        text,
        &[
            ("podinfo:podinfo", "6.x", "artifacthub"),
            ("cert-manager:cert-manager", "v1.16.2", "artifacthub"),
        ],
    );
}

#[test]
fn reads_pulumi_packages() {
    // The Pulumi project file docs' `packages` forms.
    let text = "name: example-project\nruntime: yaml\npackages:\n  static-page: https://github.com/my-org/static-page@v1.0.0\n  networking: ./components/networking\n  random:\n    source: terraform-provider\n    version: 1.4.0\n    parameters:\n      - hashicorp/random\n      - 3.7.1\n  s3-bucket:\n    source: hcl\n    version: 0.13.0\n    parameters:\n      - module\n      - terraform-aws-modules/s3-bucket/aws\n      - 4.1.2\n";
    expect(
        "Pulumi.yaml",
        text,
        &[
            ("my-org/static-page", "v1.0.0", "github"),
            ("pulumi/pulumi-terraform-provider", "1.4.0", "github"),
            ("hashicorp/random", "3.7.1", "terraform"),
            ("pulumi/pulumi-hcl", "0.13.0", "github"),
            ("terraform-aws-modules/s3-bucket/aws", "4.1.2", "terraform"),
        ],
    );
}

#[test]
fn reads_setup_action_runtime_versions() {
    let text = "jobs:\n  build:\n    steps:\n      - uses: denoland/setup-deno@v2\n        with:\n          deno-version: v2.1.4\n      - uses: actions/setup-java@v4\n        with:\n          distribution: temurin\n          java-version: '21'\n      - uses: actions/setup-dotnet@v4\n        with:\n          dotnet-version: 8.0.404\n      - uses: dtolnay/rust-toolchain@master\n        with:\n          toolchain: 1.83.0\n";
    expect(
        ".github/workflows/ci.yml",
        text,
        &[
            ("denoland/setup-deno", "v2", "github"),
            ("deno", "2.1.4", "deno-cli"),
            ("actions/setup-java", "v4", "github"),
            ("java", "21", "temurin"),
            ("actions/setup-dotnet", "v4", "github"),
            ("dotnet", "8.0.404", "dotnet-sdk"),
            ("rust", "1.83.0", "rust"),
        ],
    );
}

#[test]
fn reads_terraform_lock_providers() {
    // The lock file `terraform init` writes: developer.hashicorp.com/terraform/language/files/dependency-lock
    let text = "provider \"registry.terraform.io/hashicorp/aws\" {\n  version     = \"5.31.0\"\n  constraints = \"~> 5.0\"\n  hashes = [\n    \"h1:abc=\",\n  ]\n}\n";
    expect(
        ".terraform.lock.hcl",
        text,
        &[("hashicorp/aws", "5.31.0", "terraform")],
    );
}

#[test]
fn reads_gitlab_ci_images_and_services() {
    // docs.gitlab.com/ci/yaml/#image and #services, in their string and `name` forms.
    let text = "image: node:20.11\nservices:\n  - postgres:16.1\n  - name: redis:7.2\ntest:\n  image:\n    name: ruby:3.3\n  services:\n    - mysql:8.0\n  script: [rake]\n";
    expect(
        ".gitlab-ci.yml",
        text,
        &[
            ("node", "20.11", "docker"),
            ("postgres", "16.1", "docker"),
            ("redis", "7.2", "docker"),
            ("ruby", "3.3", "docker"),
            ("mysql", "8.0", "docker"),
        ],
    );
}

#[test]
fn reads_circleci_docker_images() {
    // circleci.com/docs/configuration-reference/#docker and #executors
    let text = "version: 2.1\nexecutors:\n  py:\n    docker:\n      - image: cimg/python:3.12.1\njobs:\n  build:\n    docker:\n      - image: cimg/node:20.11.0\n      - image: cimg/postgres:16.1\n    steps: [checkout]\n";
    expect(
        ".circleci/config.yml",
        text,
        &[
            ("cimg/python", "3.12.1", "docker"),
            ("cimg/node", "20.11.0", "docker"),
            ("cimg/postgres", "16.1", "docker"),
        ],
    );
}
