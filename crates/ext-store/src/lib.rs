//! Shared plugin persistence, grants and management. Installs never enable contributions.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
pub mod catalogue;
pub mod location;
pub mod manager;
pub mod state;
pub mod store;
pub mod unpack;
pub mod view;
pub use manager::Manager;
pub use store::Store;
pub(crate) fn state_read(path: std::path::PathBuf, max: usize) -> store::Result<Vec<u8>> {
    use std::io::Read;
    let mut b = vec![];
    std::fs::File::open(path).map_err(store::error)?.take(max as u64 + 1).read_to_end(&mut b).map_err(store::error)?;
    if b.len() > max {
        return Err(store::error("the file is too large"));
    }
    Ok(b)
}
#[cfg(test)]
mod tests;
#[cfg(test)]
mod testutil;
