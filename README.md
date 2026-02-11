# arbtt-capture-wl

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
curl -sL https://github.com/franzos/arbtt-capture-wl/releases/latest/download/arbtt-capture-wl-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv arbtt-capture-wl /usr/local/bin/
```

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

## License

GPL-3.0-only
