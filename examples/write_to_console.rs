use bevy::prelude::*;
use leafwing_terminal::{PrintTerminalLine, TerminalPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_system(write_to_terminal)
        .run();
}

fn write_to_terminal(mut terminal_line: EventWriter<PrintTerminalLine>) {
    terminal_line.send(PrintTerminalLine::new("Hello".to_string()));
}
