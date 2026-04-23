// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

//! Niri compositor backend for arbtt-wayland.

use anyhow::{Context, Result};
use niri_ipc::socket::Socket;
use niri_ipc::{Request, Response};

use super::{Backend, CaptureState, WindowInfo};

pub struct NiriBackend {
    socket: Option<Socket>,
}

impl NiriBackend {
    pub fn new() -> Self {
        Self { socket: None }
    }

    fn connect(&mut self) -> Result<&mut Socket> {
        if self.socket.is_none() {
            self.socket = Some(Socket::connect().context("failed to connect to niri socket")?);
        }
        Ok(self.socket.as_mut().unwrap())
    }

    fn capture_inner(&mut self) -> Result<CaptureState> {
        let socket = self.connect()?;

        let windows_reply = socket
            .send(Request::Windows)
            .context("failed to send Windows request")?;

        let niri_windows = match windows_reply {
            Ok(Response::Windows(windows)) => windows,
            Ok(other) => anyhow::bail!("unexpected response to Windows request: {:?}", other),
            Err(e) => anyhow::bail!("niri error on Windows request: {}", e),
        };

        let workspaces_reply = socket
            .send(Request::Workspaces)
            .context("failed to send Workspaces request")?;

        let workspaces = match workspaces_reply {
            Ok(Response::Workspaces(ws)) => ws,
            Ok(other) => anyhow::bail!("unexpected response to Workspaces request: {:?}", other),
            Err(e) => anyhow::bail!("niri error on Workspaces request: {}", e),
        };

        let desktop = workspaces
            .iter()
            .find(|ws| ws.is_focused)
            .map(|ws| {
                ws.name
                    .clone()
                    .unwrap_or_else(|| format!("workspace-{}", ws.idx))
            })
            .unwrap_or_else(|| String::from("unknown"));

        let windows: Vec<WindowInfo> = niri_windows
            .into_iter()
            .map(|w| WindowInfo {
                title: w.title.unwrap_or_default(),
                program: w.app_id.unwrap_or_default(),
                active: w.is_focused,
            })
            .collect();

        Ok(CaptureState { windows, desktop })
    }
}

impl Backend for NiriBackend {
    fn capture(&mut self) -> Result<CaptureState> {
        let result = self.capture_inner();
        if result.is_err() {
            self.socket = None;
        }
        result
    }
}
