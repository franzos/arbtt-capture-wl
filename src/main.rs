// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

mod arbtt;
mod backend;

use arbtt::ArbttImporter;
use backend::detect_backend;
use chrono::Utc;
use clap::Parser;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "arbtt-capture-wl")]
#[command(about = "arbtt capture for Wayland compositors (niri, sway)")]
struct Args {
    /// Capture interval in seconds
    #[arg(short, long, default_value = "60")]
    interval: u64,

    /// Path to arbtt log file
    #[arg(short = 'f', long)]
    logfile: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let rate = args.interval.max(1);
    let interval = Duration::from_secs(rate);

    // Ignore SIGPIPE so a dead arbtt-import doesn't kill us; the write path
    // surfaces the broken pipe as an error and triggers a restart.
    // SAFETY: called at startup, single-threaded, before any process spawn.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

    let (tx, rx) = mpsc::channel::<()>();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })?;

    let mut backend = detect_backend()?;
    let mut importer = ArbttImporter::new(args.logfile.as_deref(), rate)?;

    loop {
        let timestamp = Utc::now();

        match backend.capture() {
            Ok(state) => {
                if let Err(e) = importer.write_entry(state, timestamp) {
                    eprintln!("arbtt-capture-wl: write error: {e}");
                }
            }
            Err(e) => {
                eprintln!("arbtt-capture-wl: capture error: {e}");
            }
        }

        match rx.recv_timeout(interval) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }

    Ok(())
}
