//! RillExport crate.

#![warn(missing_docs)]

mod actors;
mod config;

use actors::embedded_node::EmbeddedNode;
use anyhow::Error;

metacrate::meta!();

// TODO: Remove env vars here
mod env {
    use std::env::var;
    use std::path::PathBuf;

    pub fn config() -> PathBuf {
        var("RILL_CONFIG")
            .unwrap_or_else(|_| "rill.toml".into())
            .into()
    }

    pub fn ui() -> Option<String> {
        var("RILL_UI").ok()
    }
}

/// The standalone server that provides access to metrics in different ways.
pub struct RillExport {
    _scoped_to_drop: meio::thread::ScopedRuntime,
}

impl RillExport {
    /// Starts an exporting server.
    pub fn start() -> Result<Self, Error> {
        let actor = EmbeddedNode::new();
        let scoped = meio::thread::spawn(actor)?;
        Ok(Self {
            _scoped_to_drop: scoped,
        })
    }
}
