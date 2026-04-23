// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

use crate::backend::{CaptureState, WindowInfo};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};

#[derive(Serialize)]
struct ArbttEntry {
    date: String,
    rate: u64,
    inactive: u64,
    windows: Vec<WindowInfo>,
    desktop: String,
}

/// Manages arbtt-import subprocess
pub struct ArbttImporter {
    child: Option<Child>,
    logfile: Option<String>,
    rate: u64,
}

impl ArbttImporter {
    pub fn new(logfile: Option<&str>, rate: u64) -> Result<Self> {
        if let Some(path) = logfile {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create directory {}", parent.display()))?;
            }
        } else if let Some(home) = std::env::var_os("HOME") {
            let arbtt_dir = Path::new(&home).join(".arbtt");
            fs::create_dir_all(&arbtt_dir)
                .with_context(|| format!("failed to create {}", arbtt_dir.display()))?;
        }

        let mut importer = Self {
            child: None,
            logfile: logfile.map(String::from),
            rate,
        };
        importer.spawn()?;
        Ok(importer)
    }

    fn spawn(&mut self) -> Result<()> {
        let mut cmd = Command::new("arbtt-import");
        cmd.args(["--format", "JSON", "--append"]);
        if let Some(ref path) = self.logfile {
            cmd.args(["--logfile", path]);
        }
        cmd.stdin(Stdio::piped());
        // SAFETY: pre_exec runs in the child after fork, before exec. We only
        // call async-signal-safe libc::signal to restore SIGPIPE default, so the
        // child doesn't inherit our parent-process SIG_IGN disposition.
        unsafe {
            cmd.pre_exec(|| {
                libc::signal(libc::SIGPIPE, libc::SIG_DFL);
                Ok(())
            });
        }

        let child = cmd
            .spawn()
            .context("failed to start arbtt-import (is arbtt installed?)")?;
        self.child = Some(child);
        Ok(())
    }

    fn ensure_running(&mut self) -> Result<()> {
        let needs_restart = match &mut self.child {
            Some(child) => child.try_wait()?.is_some(),
            None => true,
        };
        if needs_restart {
            if let Some(mut child) = self.child.take() {
                drop(child.stdin.take());
                let _ = child.wait();
            }
            eprintln!("arbtt-capture-wl: restarting arbtt-import");
            self.spawn()?;
        }
        Ok(())
    }

    fn force_restart(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            drop(child.stdin.take());
            let _ = child.kill();
            let _ = child.wait();
        }
        eprintln!("arbtt-capture-wl: restarting arbtt-import");
        self.spawn()
    }

    fn try_write(&mut self, buf: &[u8]) -> Result<()> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("arbtt-import not running"))?;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("arbtt-import stdin unavailable"))?;
        stdin.write_all(buf)?;
        stdin
            .flush()
            .context("arbtt-import write failed (process may have crashed)")?;
        Ok(())
    }

    pub fn write_entry(&mut self, state: CaptureState, timestamp: DateTime<Utc>) -> Result<()> {
        self.ensure_running()?;

        let entry = ArbttEntry {
            date: timestamp.to_rfc3339(),
            rate: self.rate * 1000,
            inactive: 0,
            windows: state.windows,
            desktop: state.desktop,
        };

        let mut buf = serde_json::to_vec(&entry)?;
        buf.push(b'\n');

        if let Err(e) = self.try_write(&buf) {
            eprintln!("arbtt-capture-wl: write failed, restarting arbtt-import: {e}");
            self.force_restart()?;
            self.try_write(&buf)?;
        }
        Ok(())
    }
}

impl Drop for ArbttImporter {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            drop(child.stdin.take());
            // Give arbtt-import up to 2s to flush and exit, then kill.
            for _ in 0..40 {
                match child.try_wait() {
                    Ok(Some(_)) => return,
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
                    Err(_) => return,
                }
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
