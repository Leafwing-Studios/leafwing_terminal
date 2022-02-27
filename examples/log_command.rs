use bevy::prelude::*;
use leafwing_terminal::{reply, AddTerminalCommand, TerminalCommand, TerminalPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_terminal_command::<LogCommand, _, _>(log_command)
        .run();
}

/// Prints given arguments to the terminal
#[derive(TerminalCommand)]
#[terminal_command(name = "log")]
struct LogCommand {
    /// Message to print
    msg: String,
    /// Number of times to print message
    num: Option<i64>,
}

fn log_command(mut log: TerminalCommand<LogCommand>) {
    if let Some(LogCommand { msg, num }) = log.take() {
        let repeat_count = num.unwrap_or(1);

        for _ in 0..repeat_count {
            reply!(log, "{msg}");
        }

        log.ok();
    }
}
