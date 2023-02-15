use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::{formatdoc, indoc};

use crate::support::command::Scarb;
use crate::support::gitx;
use crate::support::project_builder::ProjectBuilder;

#[test]
fn valid_triangle() {
    let culprit = gitx::new("culprit", |t| {
        ProjectBuilder::start()
            .name("culprit")
            .lib_cairo("fn f1() -> felt { 1 }")
            .build(&t);
    });

    let t = TempDir::new().unwrap();

    let proxy = gitx::new("proxy", |t| {
        ProjectBuilder::start()
            .name("proxy")
            .lib_cairo("fn p() -> felt { culprit::f1() }")
            .dep("culprit", &culprit)
            .build(&t);
    });

    ProjectBuilder::start()
        .name("hello")
        .version("1.0.0")
        .lib_cairo("fn hello() -> felt { proxy::p() + culprit::f1() }")
        .dep("culprit", &culprit)
        .dep("proxy", &proxy)
        .build(&t);

    Scarb::quick_snapbox()
        .arg("build")
        .current_dir(&t)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]  Updating git repository file://[..]/culprit
        [..]  Updating git repository file://[..]/proxy
        [..] Compiling hello v1.0.0 ([..])
        [..]  Finished release target(s) in [..]
        "#});
}

#[test]
fn two_revs_of_same_dep() {
    let culprit = gitx::new("culprit", |t| {
        ProjectBuilder::start()
            .name("culprit")
            .lib_cairo("fn f1() -> felt { 1 }")
            .build(&t);
    });

    culprit.checkout_branch("branchy");
    culprit.change_file("src/lib.cairo", "fn f2() -> felt { 2 }");

    let t = TempDir::new().unwrap();

    let proxy = t.child("vendor/proxy");
    ProjectBuilder::start()
        .name("proxy")
        .lib_cairo("fn p() -> felt { culprit::f2() }")
        .dep(
            "culprit",
            formatdoc! {r#"
                git = "{culprit}"
                branch = "branchy"
            "#},
        )
        .build(&proxy);

    ProjectBuilder::start()
        .name("hello")
        .version("1.0.0")
        .lib_cairo("fn hello() -> felt { proxy::p() + culprit::f1() }")
        .dep("culprit", &culprit)
        .dep("proxy", &proxy)
        .build(&t);

    Scarb::quick_snapbox()
        .arg("build")
        .current_dir(&t)
        .assert()
        .failure()
        .stdout_matches(indoc! {r#"
            [..] Updating git repository file://[..]/culprit
            error: found dependencies on the same package `culprit` coming from incompatible sources:
            source 1: git+file://[..]/culprit
            source 2: git+file://[..]/culprit?branch=branchy
        "#});
}

#[test]
fn two_revs_of_same_dep_diamond() {
    let culprit = gitx::new("culprit", |t| {
        ProjectBuilder::start()
            .name("culprit")
            .lib_cairo("fn f1() -> felt { 1 }")
            .build(&t);
    });

    culprit.checkout_branch("branchy");
    culprit.change_file("src/lib.cairo", "fn f2() -> felt { 2 }");

    let t = TempDir::new().unwrap();

    let dep1 = gitx::new("dep1", |t| {
        ProjectBuilder::start()
            .name("dep1")
            .lib_cairo("fn p() -> felt { culprit::f1() }")
            .dep("culprit", &culprit)
            .build(&t);
    });

    let dep2 = gitx::new("dep2", |t| {
        ProjectBuilder::start()
            .name("dep2")
            .lib_cairo("fn p() -> felt { culprit::f2() }")
            .dep(
                "culprit",
                formatdoc! {r#"
                    git = "{culprit}"
                    branch = "branchy"
                "#},
            )
            .build(&t);
    });

    ProjectBuilder::start()
        .name("hello")
        .version("1.0.0")
        .lib_cairo("fn hello() -> felt { dep1::p() + dep2::p() }")
        .dep("dep1", &dep1)
        .dep("dep2", &dep2)
        .build(&t);

    Scarb::quick_snapbox()
        .arg("build")
        .current_dir(&t)
        .assert()
        .failure()
        .stdout_matches(indoc! {r#"
            [..] Updating git repository file://[..]/dep1
            [..] Updating git repository file://[..]/dep2
            [..] Updating git repository file://[..]/culprit
            error: found dependencies on the same package `culprit` coming from incompatible sources:
            source 1: git+file://[..]/culprit
            source 2: git+file://[..]/culprit?branch=branchy
        "#});
}