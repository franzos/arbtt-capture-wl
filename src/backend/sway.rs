// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

//! Sway compositor backend for arbtt-capture-wl.

use anyhow::{Context, Result};
use swayipc::{Connection, Node, NodeType};

use super::{Backend, CaptureState, WindowInfo};

/// Backend implementation for the sway Wayland compositor.
pub struct SwayBackend {
    connection: Option<Connection>,
}

impl SwayBackend {
    /// Create a new sway backend.
    pub fn new() -> Result<Self> {
        Ok(Self { connection: None })
    }

    /// Ensure connection is established, reconnecting if necessary.
    fn connect(&mut self) -> Result<&mut Connection> {
        if self.connection.is_none() {
            self.connection =
                Some(Connection::new().context("failed to connect to sway socket")?);
        }
        Ok(self.connection.as_mut().unwrap())
    }

    /// Recursively collect all windows from the tree.
    fn collect_windows(node: &Node, windows: &mut Vec<WindowInfo>) {
        // A node is a window if it has a Con type and an app_id or window_properties
        if node.node_type == NodeType::Con || node.node_type == NodeType::FloatingCon {
            let has_window_info =
                node.app_id.is_some() || node.window_properties.is_some() || node.name.is_some();

            if has_window_info && node.pid.is_some() {
                let program = node
                    .app_id
                    .clone()
                    .or_else(|| {
                        node.window_properties
                            .as_ref()
                            .and_then(|p| p.class.clone())
                    })
                    .unwrap_or_default();

                let title = node.name.clone().unwrap_or_default();

                windows.push(WindowInfo {
                    title,
                    program,
                    active: node.focused,
                });
            }
        }

        // Recurse into child nodes
        for child in &node.nodes {
            Self::collect_windows(child, windows);
        }
        for child in &node.floating_nodes {
            Self::collect_windows(child, windows);
        }
    }

    fn capture_inner(&mut self) -> Result<CaptureState> {
        let conn = self.connect()?;

        // Get the window tree
        let tree = conn.get_tree().context("failed to get window tree")?;

        // Collect all windows
        let mut windows = Vec::new();
        Self::collect_windows(&tree, &mut windows);

        // Get workspaces to find focused one
        let workspaces = conn.get_workspaces().context("failed to get workspaces")?;

        let desktop = workspaces
            .iter()
            .find(|ws| ws.focused)
            .map(|ws| ws.name.clone())
            .unwrap_or_else(|| String::from("unknown"));

        Ok(CaptureState { windows, desktop })
    }
}

impl Backend for SwayBackend {
    fn capture(&mut self) -> Result<CaptureState> {
        let result = self.capture_inner();
        if result.is_err() {
            // Disconnect so next capture attempt reconnects
            self.connection = None;
        }
        result
    }
}
