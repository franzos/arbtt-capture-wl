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

## Supported compositors

- sway (via `$SWAYSOCK`)
- niri (via `$NIRI_SOCKET`)

## License

GPL-3.0-only
