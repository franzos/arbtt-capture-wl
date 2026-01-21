// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Franz Geffke <mail@gofranz.com>

mod arbtt;
mod backend;

use arbtt::ArbttImporter;
use backend::detect_backend;
use chrono::Utc;
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "arbtt-wayland")]
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

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let mut backend = detect_backend()?;
    let mut importer = ArbttImporter::new(args.logfile.as_deref(), args.interval)?;

    while running.load(Ordering::SeqCst) {
        let state = backend.capture()?;
        let timestamp = Utc::now();
        importer.write_entry(state, timestamp)?;

        for _ in 0..(args.interval * 10) {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    Ok(())
}
