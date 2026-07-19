use versionlens_model::Ecosystem;

use super::ResponseRequest;
use crate::response::cpp::latest_cpp_text_version;
use crate::response::cran::latest_cran_version;
use crate::response::go::latest_go_version;
use crate::response::haxelib::latest_haxelib_version;
use crate::response::helm::latest_helm_version;
use crate::response::julia::latest_julia_version;
use crate::response::luarocks::latest_luarocks_version;
use crate::response::opam::latest_opam_version;
use crate::response::python::latest_python_version;
use crate::response::xml::latest_maven_version;
use versionlens_model::Ecosystem::{
    Cpp, Cran, Go, Haxelib, Helm, Julia, LuaRocks, Maven, Opam, Python,
};

pub(super) fn latest_text_response(
    ecosystem: Ecosystem,
    body: &str,
    request: &TextResponseRequest<'_>,
) -> Option<String> {
    match ecosystem {
        Cpp => latest_cpp_text_version(body, request.include_prereleases, request.prerelease_tags),
        Cran => latest_cran_version(
            body,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        ),
        Go => latest_go_version(body, request.include_prereleases, request.prerelease_tags),
        Haxelib => latest_haxelib_version(
            body,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        ),
        Helm => latest_helm_version(
            body,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        ),
        Julia => latest_julia_version(body, request.include_prereleases, request.prerelease_tags),
        LuaRocks => latest_luarocks_version(
            body,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        ),
        Maven => latest_maven_version(body, request.include_prereleases, request.prerelease_tags),
        Opam => latest_opam_version(body, request.include_prereleases, request.prerelease_tags),
        Python => latest_python_version(body, request.include_prereleases, request.prerelease_tags),
        _ => None,
    }
}

type TextResponseRequest<'a> = ResponseRequest<'a>;
