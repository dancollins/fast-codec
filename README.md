# fast-codec

A desktop scratchpad for encodings: type in one box, the other box shows the result,
in either direction. It is meant to be on screen the moment you press its hotkey.

## Build

Needs a [Rust toolchain](https://rustup.rs). No system libraries are needed to build on Linux.

```sh
cargo build --release
```

The binary is `target/release/fast-codec` (`fast-codec.exe` on Windows).

## Open it with a key

fast-codec has no hotkey of its own; something else launches it. On Windows that is
[AutoHotkey v2](https://www.autohotkey.com/). Launching from AutoHotkey also avoids
Windows opening the window behind the one you are in. Put this in a script in
`shell:startup`, with your own key and path:

```ahk
#Requires AutoHotkey v2.0
#SingleInstance Force

^!b:: RunOrActivate("ahk_exe fast-codec.exe", "C:\Tools\fast-codec.exe")

RunOrActivate(winTitle, target) {
    if !WinExist(winTitle) {
        Run target
        if !WinWait(winTitle, , 5)
            return
    }
    WinActivate
}
```

On Linux, bind the key in your desktop environment's keyboard shortcuts.

## Layout

| Path | What |
|---|---|
| `crates/app` | The GUI: Slint markup in `ui/`, Rust in `src/` |
