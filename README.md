# leafwing_terminal

A simple virtual terminal for Bevy games with support for argument parsing.

<p align="center">
  <img src="https://raw.githubusercontent.com/tqwewe/bevy-terminal/main/doc/screenshot.png" width="100%">
</p>

## Usage

Add `TerminalPlugin` and optionally the resource `TerminalConfiguration`.

```rust
use bevy::prelude::*;
use leafwing_terminal::{TerminalConfiguration, TerminalPlugin};

fn main() {
    App::new()
        .add_plugin(TerminalPlugin)
        .insert_resource(TerminalConfiguration {
            // override config here
            ..Default::default()
        });
}
```

Create a terminal command struct and system and add it to your app with `.add_terminal_command`.

Add [doc comments](https://doc.rust-lang.org/rust-by-example/meta/doc.html#doc-comments) to your command to provide help information in the terminal.

```rust
use bevy::prelude::*;
use leafwing_terminal::{reply, AddTerminalCommand, TerminalCommand, TerminalPlugin};

fn main() {
    App::new()
        .add_plugin(TerminalPlugin)
        .add_terminal_command::<ExampleCommand, _, _>(example_command);
}

/// Example command
#[derive(TerminalCommand)]
#[terminal_command(name = "example")]
struct ExampleCommand {
    /// Some message
    msg: String,
}

fn example_command(mut log: TerminalCommand<ExampleCommand>) {
    if let Some(ExampleCommand { msg }) = log.take() {
        // handle command
    }
}
```

Examples can be found in the [/examples](examples) directory.

```bash
cargo run --example log_command
```

- [log_command](/examples/log_command.rs)
- [raw_commands](/examples/raw_commands.rs)
- [write_to_terminal](/examples/write_to_terminal.rs)

## wasm

Should work in wasm, but you need to disable default features.
