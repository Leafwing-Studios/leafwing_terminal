use bevy::app::AppExit;
use bevy::prelude::*;

use crate as leafwing_terminal;
use crate::TerminalCommand;

/// Exits the app
#[derive(TerminalCommand)]
#[terminal_command(name = "exit")]
pub(crate) struct ExitCommand;

pub(crate) fn exit_command(
    mut exit: TerminalCommand<ExitCommand>,
    mut exit_writer: EventWriter<AppExit>,
) {
    if exit.take().is_some() {
        exit_writer.send(AppExit);
        exit.ok();
    }
}
