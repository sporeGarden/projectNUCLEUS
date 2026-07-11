pub mod depot;
pub mod dns;
pub mod forge;
pub mod http;
pub mod mesh;
pub mod tls;

use crate::check::CheckResult;

pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    tls::run(target, results);
    http::run(target, results);
    depot::run(target, results);
    forge::run(target, results);
    dns::run(target, results);
    mesh::run(target, results);
}
