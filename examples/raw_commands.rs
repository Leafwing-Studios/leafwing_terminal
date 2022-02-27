use bevy::prelude::*;
use leafwing_terminal::{TerminalCommandEntered, TerminalPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_system(raw_commands)
        .run();
}

fn raw_commands(mut terminal_commands: EventReader<TerminalCommandEntered>) {
    for TerminalCommandEntered { command, args } in terminal_commands.iter() {
        println!(r#"Entered command "{command}" with args {:#?}"#, args);
    }
}
