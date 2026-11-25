//! Infrastructure and environment manifests: Terraform modules, Terragrunt, Packer, helmfile,
//! Argo CD, Flux, Vagrant, Snapcraft, Flatpak, proto, aqua, Devbox, and Spack.

use super::extract;
use crate::catalog::file_type_of;

/// Each dependency's name, requirement, and registry, for a document opened at `path`.
fn found(path: &str, text: &str) -> Vec<(String, String, &'static str)> {
    let file = file_type_of(path, text).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .map(|d| {
            assert_eq!(text[d.span.clone()], d.req, "span of {}", d.name);
            (d.name, d.req, d.registry)
        })
        .collect()
}

fn expect(path: &str, text: &str, want: &[(&str, &str, &str)]) {
    let want: Vec<_> = want
        .iter()
        .map(|(n, r, g)| ((*n).to_owned(), (*r).to_owned(), *g))
        .collect();
    assert_eq!(found(path, text), want, "{path}");
}

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
