// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

mod niri;
mod sway;

pub use niri::NiriBackend;
pub use sway::SwayBackend;

use anyhow::Result;

/// Window information captured from compositor
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub program: String,
    pub active: bool,
}

/// Capture state from compositor
#[derive(Debug, Clone)]
pub struct CaptureState {
    pub windows: Vec<WindowInfo>,
    pub desktop: String,
}

/// Backend trait for compositor-specific implementations
pub trait Backend {
    fn capture(&mut self) -> Result<CaptureState>;
}

/// Detect and create appropriate backend
pub fn detect_backend() -> Result<Box<dyn Backend>> {
    if std::env::var("SWAYSOCK").is_ok() {
        Ok(Box::new(SwayBackend::new()?))
    } else if std::env::var("NIRI_SOCKET").is_ok() {
        Ok(Box::new(NiriBackend::new()?))
    } else {
        anyhow::bail!("No supported compositor detected (checked: SWAYSOCK, NIRI_SOCKET)")
    }
}
