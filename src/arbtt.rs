// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

use crate::backend::CaptureState;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};

/// Window entry for arbtt JSON format
#[derive(Serialize)]
struct ArbttWindow {
    title: String,
    program: String,
    active: bool,
}

/// Log entry for arbtt JSON format
#[derive(Serialize)]
struct ArbttEntry {
    date: String,
    rate: u64,
    inactive: u64,
    windows: Vec<ArbttWindow>,
    desktop: String,
}

/// Manages arbtt-import subprocess
pub struct ArbttImporter {
    child: Child,
    rate: u64,
}

impl ArbttImporter {
    pub fn new(logfile: Option<&str>, rate: u64) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(path) = logfile {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create directory {:?}", parent))?;
            }
        } else {
            // Default location: ~/.arbtt/
            if let Some(home) = std::env::var_os("HOME") {
                let arbtt_dir = Path::new(&home).join(".arbtt");
                fs::create_dir_all(&arbtt_dir)
                    .with_context(|| format!("failed to create {:?}", arbtt_dir))?;
            }
        }

        let mut cmd = Command::new("arbtt-import");
        cmd.args(["--format", "JSON", "--append"]);
        if let Some(path) = logfile {
            cmd.args(["--logfile", path]);
        }
        cmd.stdin(Stdio::piped());

        let child = cmd.spawn()?;
        Ok(Self { child, rate })
    }

    pub fn write_entry(&mut self, state: CaptureState, timestamp: DateTime<Utc>) -> Result<()> {
        // Check if arbtt-import is still running
        if let Some(status) = self.child.try_wait()? {
            anyhow::bail!("arbtt-import exited unexpectedly with {}", status);
        }

        let entry = ArbttEntry {
            date: timestamp.to_rfc3339(),
            rate: self.rate * 1000,
            inactive: 0,
            windows: state
                .windows
                .into_iter()
                .map(|w| ArbttWindow {
                    title: w.title,
                    program: w.program,
                    active: w.active,
                })
                .collect(),
            desktop: state.desktop,
        };

        let stdin = self.child.stdin.as_mut().ok_or_else(|| {
            anyhow::anyhow!("arbtt-import stdin unavailable")
        })?;

        let mut buf = serde_json::to_vec(&entry)?;
        buf.push(b'\n');
        stdin.write_all(&buf)?;
        stdin.flush().context("arbtt-import write failed (process may have crashed)")?;

        Ok(())
    }
}

impl Drop for ArbttImporter {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}
