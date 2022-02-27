use bevy::prelude::*;

use crate as leafwing_terminal;
use crate::terminal::TerminalState;
use crate::TerminalCommand;

/// Clears the terminal
#[derive(TerminalCommand)]
#[terminal_command(name = "clear")]
pub(crate) struct ClearCommand;

pub(crate) fn clear_command(
    mut clear: TerminalCommand<ClearCommand>,
    mut state: ResMut<TerminalState>,
) {
    if clear.take().is_some() {
        state.scrollback.clear();
    }
}
