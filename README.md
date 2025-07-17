# `swayalt@0.1`

An automatic alternating tile layout for Sway, written in Rust with love.

Note that while `swayalt` was designed for Sway, it is compatible with i3.

## Usage

Run `swayalt` as a background process in your window manager configuration, or
in a terminal to take it for a spin. There is no CLI or output - `swayalt`
relies solely on the `SWAYSOCK` and `I3SOCK` environment variables.

## Performance

`swayalt` has an extremely low memory profile (comparable to `swaymsg`), and
uses two connections to the IPC socket to avoid continually spawning
subprocesses. Therefore, its impact on your system should be negligible.

## Caveats

- `swayalt` will attempt to spawn `sway` or `i3` if neither of the socket
  environment variables are set. This is a flaw inherited from [`swayipc`].
- `swayalt` may produce confusing errors if the socket does not exist or has
  incorrect permissions. This will be fixed in the future.

[`swayipc`]: https://github.com/JayceFayne/swayipc-rs

## Credits

Thank you to [@JayceFayne] for authoring [`swayipc`].  
Thank you to the [`swaywm`] contributors for creating Sway.  
Thank you to [@megatank58] for verifying that `swayalt` runs on i3.  

[@JayceFayne]: https://github.com/JayceFayne
[`swaywm`]: https://github.com/swaywm
[@megatank58]: https://github.com/megatank58
