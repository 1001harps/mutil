```
midi utility cli

Usage: mutil [OPTIONS] <COMMAND>

Commands:
  devices         Get list of midi devices
  input-devices   Get list of midi input devices
  output-devices  Get list of midi output devices
  stream          Open stream of incoming midi messages
  note-on         Send midi note on message
  note-off        Send midi note off message
  trig            Send midi note on message followed by note off message
  help            Print this message or the help of the given subcommand(s)

Options:
  -c, --channel <CHANNEL>  Midi channel number
  -d, --device <DEVICE>    Midi device id
  -h, --help               Print help
  -V, --version            Print version
```

---

Dependencies:

- [portmidi](https://portmedia.sourceforge.net/portmidi/)
