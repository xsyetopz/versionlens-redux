use crate::docker::{parse_docker_compose_yaml, parse_dockerfile};
use crate::gemfile::parse_gemfile;
use crate::go_mod::parse_go_mod;
use crate::model::Dependency;
use crate::requirements_txt::parse_requirements_txt;

pub(super) fn parse_docker_compose_document(text: &str, _paths: &[&str]) -> Vec<Dependency> {
    parse_docker_compose_yaml(text)
}

pub(super) fn parse_dockerfile_document(text: &str, _paths: &[&str]) -> Vec<Dependency> {
    parse_dockerfile(text)
}

pub(super) fn parse_gemfile_document(text: &str, _paths: &[&str]) -> Vec<Dependency> {
    parse_gemfile(text)
}

pub(super) fn parse_go_mod_document(text: &str, _paths: &[&str]) -> Vec<Dependency> {
    parse_go_mod(text)
}

pub(super) fn parse_requirements_document(text: &str, _paths: &[&str]) -> Vec<Dependency> {
    parse_requirements_txt(text)
}
