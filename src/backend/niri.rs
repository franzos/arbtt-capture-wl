// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

//! Niri compositor backend for arbtt-wayland.

use anyhow::{Context, Result};
use niri_ipc::socket::Socket;
use niri_ipc::{Request, Response};

use super::{Backend, CaptureState, WindowInfo};

/// Backend implementation for the niri Wayland compositor.
pub struct NiriBackend {
    socket: Socket,
}

impl NiriBackend {
    /// Create a new niri backend by connecting to the niri IPC socket.
    ///
    /// The socket path is read from the `NIRI_SOCKET` environment variable.
    pub fn new() -> Result<Self> {
        let socket = Socket::connect().context("failed to connect to niri socket")?;
        Ok(Self { socket })
    }
}

impl Backend for NiriBackend {
    fn capture(&mut self) -> Result<CaptureState> {
        // Get all windows
        let windows_reply = self
            .socket
            .send(Request::Windows)
            .context("failed to send Windows request")?;

        let niri_windows = match windows_reply {
            Ok(Response::Windows(windows)) => windows,
            Ok(other) => anyhow::bail!("unexpected response to Windows request: {:?}", other),
            Err(e) => anyhow::bail!("niri error on Windows request: {}", e),
        };

        // Get focused window
        let focused_reply = self
            .socket
            .send(Request::FocusedWindow)
            .context("failed to send FocusedWindow request")?;

        let focused_window_id = match focused_reply {
            Ok(Response::FocusedWindow(Some(w))) => Some(w.id),
            Ok(Response::FocusedWindow(None)) => None,
            Ok(other) => anyhow::bail!("unexpected response to FocusedWindow request: {:?}", other),
            Err(e) => anyhow::bail!("niri error on FocusedWindow request: {}", e),
        };

        // Get workspaces to determine focused workspace name
        let workspaces_reply = self
            .socket
            .send(Request::Workspaces)
            .context("failed to send Workspaces request")?;

        let workspaces = match workspaces_reply {
            Ok(Response::Workspaces(ws)) => ws,
            Ok(other) => anyhow::bail!("unexpected response to Workspaces request: {:?}", other),
            Err(e) => anyhow::bail!("niri error on Workspaces request: {}", e),
        };

        // Find focused workspace name
        let desktop = workspaces
            .iter()
            .find(|ws| ws.is_focused)
            .map(|ws| {
                ws.name
                    .clone()
                    .unwrap_or_else(|| format!("workspace-{}", ws.idx))
            })
            .unwrap_or_else(|| String::from("unknown"));

        // Convert windows to WindowInfo
        let windows: Vec<WindowInfo> = niri_windows
            .iter()
            .map(|w| WindowInfo {
                title: w.title.clone().unwrap_or_default(),
                program: w.app_id.clone().unwrap_or_default(),
                active: Some(w.id) == focused_window_id,
            })
            .collect();

        Ok(CaptureState { windows, desktop })
    }
}
