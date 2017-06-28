obsidian
========

A somewhat lightweight bar written in Rust. Gtk3 is used for layouting and cairo
context creation.

Currently only compatible with i3. Connects to MPD to display the currently
playing song.

Screenshot
----------

![a screenshot](http://dark.red/s/NKCgr5.png)

Configuration
-------------

Config goes in `~/.config/obsidian/config.toml`

Everything is optional and has sensible defaults. (see `src/default_config.toml`)

```toml
# The components to display on the right side of the bar
status_items = [ 'memory', 'load', 'battery', 'clock' ]

# Connection details for MPD
[mpd]
host = "192.168.0.123"
port = 6600

# Run shell commands when right-clicking the corresponding third of the bar
[launch]
left   = "influence"
middle = "vinyl"
right  = "calendar"

# Override some colors (#rrggbbaa, optional alpha)
[colors]
red    = "#e84f4f"
green  = "#b8d68c"
yellow = "#e1aa5d"
blue   = "#7dc1cf"
```
