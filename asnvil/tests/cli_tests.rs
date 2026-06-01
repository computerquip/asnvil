use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn make_test_asn1() -> &'static str {
    r#"TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
        Person ::= SEQUENCE {
            name    UTF8String,
            age     INTEGER,
            active  BOOLEAN DEFAULT TRUE
        }
    END;
    "#
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ASN.1 Compiler"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_cli_no_input() {
    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No input files specified"));
}

#[test]
fn test_cli_invalid_file() {
    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg("nonexistent.asn1");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read input file"));
}

#[test]
fn test_cli_compile_valid() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(temp.path());
    cmd.assert().success();

    let output = temp.path().join("TestModule.py");
    assert!(output.exists(), "Generated file should exist");
    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("class Person"));
    assert!(content.contains("encode_der"));
    assert!(content.contains("decode_der"));
}

#[test]
fn test_cli_compile_invalid_syntax() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("bad.asn1");
    fs::write(&input, "INVALID ASN.1 GARBAGE @#$%").unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(temp.path());
    cmd.assert().failure();
}

#[test]
fn test_cli_output_directory_created() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let out_dir = temp.path().join("nested").join("output");
    assert!(!out_dir.exists());

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(&out_dir);
    cmd.assert().success();

    assert!(out_dir.exists());
    assert!(out_dir.join("TestModule.py").exists());
}

#[test]
fn test_cli_unsupported_language() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(temp.path()).arg("--lang").arg("rust");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported target language"));
}

#[test]
fn test_cli_verbose() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(temp.path()).arg("--verbose");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing"));
}

#[test]
fn test_cli_quiet() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(temp.path()).arg("--quiet");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated").not());
}

#[test]
fn test_cli_print_ir() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("--print-ir");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AsnModule"));
}

#[test]
fn test_cli_print_ast() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("--print-ast");
    cmd.assert().success();
}

#[test]
fn test_cli_emit_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("test.asn1");
    fs::write(&input, make_test_asn1()).unwrap();

    let mut cmd = Command::cargo_bin("asnvil").unwrap();
    cmd.arg(&input).arg("-o").arg(temp.path()).arg("--emit-runtime");
    cmd.assert().success();

    let runtime = temp.path().join("asnvil_runtime");
    assert!(runtime.exists());
    assert!(runtime.join("__init__.py").exists());
    assert!(runtime.join("ber.py").exists());
    assert!(runtime.join("der.py").exists());
}
