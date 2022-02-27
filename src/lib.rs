#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
pub use leafwing_terminal_derive::TerminalCommand;
pub use leafwing_terminal_parser::{Value, ValueRawOwned};

use crate::commands::clear::{clear_command, ClearCommand};
use crate::commands::exit::{exit_command, ExitCommand};
use crate::commands::help::{help_command, HelpCommand};
use crate::terminal::{receive_terminal_line, TerminalState};
pub use crate::terminal::{
    AddTerminalCommand, CommandArgInfo, CommandArgs, CommandHelp, CommandInfo, CommandName,
    PrintTerminalLine, TerminalCommand, TerminalCommandEntered, TerminalConfiguration,
};
use crate::ui::terminal_ui;
pub use crate::value::{FromValue, FromValueError, ValueType};

mod commands;
mod macros;
mod terminal;
mod ui;
mod value;

/// Terminal plugin.
pub struct TerminalPlugin;

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerminalConfiguration>()
            .init_resource::<TerminalState>()
            .add_event::<TerminalCommandEntered>()
            .add_event::<PrintTerminalLine>()
            .add_plugin(EguiPlugin)
            .add_terminal_command::<ClearCommand, _, _>(clear_command)
            .add_terminal_command::<ExitCommand, _, _>(exit_command)
            .add_terminal_command::<HelpCommand, _, _>(help_command)
            .add_system(terminal_ui)
            .add_system(receive_terminal_line);
    }
}
