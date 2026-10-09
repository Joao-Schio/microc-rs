//! End-to-end tests for the compiler executable and generated LLVM IR.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("microc-rs-cli-{}-{id}", std::process::id()));
        fs::create_dir(&directory).expect("create isolated test directory");
        Self { directory }
    }

    fn source(&self, content: &str) -> PathBuf {
        let path = self.directory.join("input.mc");
        fs::write(&path, content).expect("write test source");
        path
    }

    fn run(&self, path: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_microc-rs"))
            .arg(path)
            .output()
            .expect("run MicroC-RS")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).expect("clean up CLI test fixture");
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("compiler diagnostics are UTF-8")
}

#[test]
fn emits_llvm_ir_to_source_adjacent_file() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { return (3 + 4) * (8 - 2); }");

    let output = fixture.run(&source);

    assert!(output.status.success(), "{}", stderr(&output));
    let ir_path = source.with_extension("ll");
    assert_eq!(
        fs::read_to_string(&ir_path).unwrap(),
        concat!(
            "define i32 @main() {\n",
            "entry:\n",
            "  %t0 = add i32 3, 4\n",
            "  %t1 = sub i32 8, 2\n",
            "  %t2 = mul i32 %t0, %t1\n",
            "  ret i32 %t2\n",
            "}\n"
        )
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Generated LLVM IR: {}\n", ir_path.display())
    );
}

#[test]
fn rejects_missing_source_argument() {
    let output = Command::new(env!("CARGO_BIN_EXE_microc-rs"))
        .output()
        .expect("run MicroC-RS");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("usage: microc-rs"));
}

#[test]
fn rejects_extra_source_argument() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { return 0; }");
    let output = Command::new(env!("CARGO_BIN_EXE_microc-rs"))
        .arg(&source)
        .arg(&source)
        .output()
        .expect("run MicroC-RS");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("usage: microc-rs"));
    assert!(!source.with_extension("ll").exists());
}

#[test]
fn reports_failure_opening_missing_source() {
    let fixture = Fixture::new();
    let path = fixture.directory.join("does-not-exist.mc");
    let output = fixture.run(&path);

    assert!(!output.status.success());
    assert!(stderr(&output).starts_with("MicroC error: "));
    assert!(!path.with_extension("ll").exists());
}

#[test]
fn reports_lexical_error_without_writing_ir() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { return @; }");
    let output = fixture.run(&source);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unexpected character '@'"));
    assert!(!source.with_extension("ll").exists());
}

#[test]
fn reports_parser_error_without_writing_ir() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { return 42 }");
    let output = fixture.run(&source);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("expected"));
    assert!(!source.with_extension("ll").exists());
}

#[test]
fn reports_semantic_error_without_writing_ir() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { return missing; }");
    let output = fixture.run(&source);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("undeclared variable 'missing'"));
    assert!(!source.with_extension("ll").exists());
}

#[test]
fn reports_unsupported_backend_feature_without_writing_ir() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { print(42); return 0; }");
    let output = fixture.run(&source);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("LLVM backend does not yet support statement"));
    assert!(!source.with_extension("ll").exists());
}

#[test]
fn reports_missing_return_without_writing_ir() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { ; }");
    let output = fixture.run(&source);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("function 'main' does not return a value"));
    assert!(!source.with_extension("ll").exists());
}

#[test]
fn reports_failure_writing_llvm_ir() {
    let fixture = Fixture::new();
    let source = fixture.source("int main() { return 42; }");
    // A directory at the destination makes fs::write fail on every platform.
    let destination = source.with_extension("ll");
    fs::create_dir(&destination).unwrap();

    let output = fixture.run(&source);

    assert!(!output.status.success());
    assert!(stderr(&output).starts_with("MicroC error: "));
    assert!(destination.is_dir());
}
