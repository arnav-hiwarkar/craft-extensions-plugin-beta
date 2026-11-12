//! Small schema-two archive fixtures.
#![allow(clippy::unwrap_used)]
use std::io::{Cursor, Write};
pub fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(Cursor::new(vec![]));
    for (name, b) in entries {
        w.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        w.write_all(b).unwrap();
    }
    w.finish().unwrap().into_inner()
}
pub fn package(version: &str, permission: &str, payload: &[u8]) -> Vec<u8> {
    let text = format!(
        r#"schema = 2
id = "org.example.test"
name = "Test"
version = "{version}"
license = "MIT"
source = "https://github.com/example/test"
summary = "Test package"
apps = ["photocraft"]
[[contribution]]
id = "run"
kind = "command"
contract = "app.session@1.0"
label = "Run"
module = "run.wasm"
permissions = [{permission}]
"#
    );
    zip_of(&[(craft_ext_core::manifest::MANIFEST_NAME, text.as_bytes()), ("run.wasm", payload)])
}
