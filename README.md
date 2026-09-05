# arbtt-capture-wl

[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/franzos/arbtt-capture-wl/badge)](https://scorecard.dev/viewer/?uri=github.com/franzos/arbtt-capture-wl)

arbtt capture for Wayland compositors (niri, sway).

## Usage

```bash
# Start capturing (60s interval, writes to ~/.arbtt/capture.log)
arbtt-capture-wl

# Custom interval
arbtt-capture-wl -i 30

# Custom logfile
arbtt-capture-wl -f /path/to/capture.log
```

## View stats

Requires `~/.arbtt/categorize.cfg` - see [example config](https://github.com/nomeata/arbtt/blob/master/categorize.cfg).

```bash
arbtt-stats
arbtt-dump
```

## Install

**Pre-built binaries:**

Download the latest release from [GitHub Releases](https://github.com/franzos/arbtt-capture-wl/releases):

```bash
# Linux (x86_64) - binary
base=https://github.com/franzos/arbtt-capture-wl/releases/latest/download
curl -sLO $base/arbtt-capture-wl-x86_64-unknown-linux-gnu.tar.gz
curl -sLO $base/arbtt-capture-wl-x86_64-unknown-linux-gnu.tar.gz.sha256
sha256sum -c arbtt-capture-wl-x86_64-unknown-linux-gnu.tar.gz.sha256
tar xzf arbtt-capture-wl-x86_64-unknown-linux-gnu.tar.gz
sudo mv arbtt-capture-wl /usr/local/bin/
```

Release assets carry a build provenance attestation; verify with `gh attestation verify <file> --repo franzos/arbtt-capture-wl`.

**Packages:**

`.deb` and `.rpm` packages are also available on the [Releases](https://github.com/franzos/arbtt-capture-wl/releases) page.

```bash
# Debian/Ubuntu
sudo dpkg -i arbtt-capture-wl_*.deb

# Fedora/RHEL
sudo rpm -i arbtt-capture-wl-*.rpm
```

## Supported compositors

- sway (via `$SWAYSOCK`)
- niri (via `$NIRI_SOCKET`)

## Limitations

Idle time isn't detected. Every sample is recorded as active, so `$idle` conditions in `categorize.cfg` never match and time spent away from an unlocked machine counts as active. Neither compositor exposes idle time over its IPC socket; that would need an `ext-idle-notify-v1` listener.

## License

GPL-3.0-only
