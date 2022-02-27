use bevy::prelude::*;

use crate as leafwing_terminal;
use crate::console::ConsoleState;
use crate::ConsoleCommand;

/// Clears the console
#[derive(ConsoleCommand)]
#[console_command(name = "clear")]
pub(crate) struct ClearCommand;

pub(crate) fn clear_command(
    mut clear: ConsoleCommand<ClearCommand>,
    mut state: ResMut<ConsoleState>,
) {
    if clear.take().is_some() {
        state.scrollback.clear();
    }
}
