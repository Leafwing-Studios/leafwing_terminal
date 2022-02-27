use bevy::prelude::*;

use crate as leafwing_terminal;
use crate::{reply, CommandInfo, TerminalCommand, TerminalConfiguration};

/// Prints available arguments and usage
#[derive(TerminalCommand)]
#[terminal_command(name = "help")]
pub(crate) struct HelpCommand {
    /// Help for a given command
    command: Option<String>,
}

pub(crate) fn help_command(
    mut help: TerminalCommand<HelpCommand>,
    config: Res<TerminalConfiguration>,
) {
    match help.take() {
        Some(HelpCommand { command: Some(cmd) }) => match config.commands.get(cmd.as_str()) {
            Some(Some(command_info)) => {
                help.reply(command_info.help_text());
            }
            Some(None) => {
                reply!(help, "Help not available for command '{}'", cmd);
            }
            None => {
                reply!(help, "Command '{}' does not exist", cmd);
            }
        },
        Some(HelpCommand { command: None }) => {
            reply!(help, "Available commands:");
            let longest_command_name = config
                .commands
                .keys()
                .map(|name| name.len())
                .max()
                .unwrap_or(0);
            for (name, cmd) in &config.commands {
                let mut line = format!("  {name}{}", " ".repeat(longest_command_name - name.len()));
                if let Some(CommandInfo {
                    description: Some(description),
                    ..
                }) = cmd
                {
                    line.push_str(&format!(" - {description}"));
                }
                help.reply(line);
            }
            help.reply("");
        }
        None => {}
    }
}
