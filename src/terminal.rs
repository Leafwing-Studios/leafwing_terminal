use std::collections::{BTreeMap, VecDeque};
use std::marker::PhantomData;
use std::{fmt::Write, mem};

use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::{
    ecs::event::{EventReaderState, EventWriterState, Events},
    ecs::system::{
        LocalState, ResMutState, ResState, Resource, SystemMeta, SystemParam, SystemParamFetch,
        SystemParamState,
    },
    prelude::*,
};
use leafwing_terminal_parser::ValueRawOwned;

use crate::FromValueError;

/// Terminal command name.
///
/// # Example
///
/// `log "hello"`
///
/// ```
/// # use leafwing_terminal::CommandName;
/// #
/// struct LogCommand;
///
/// impl CommandName for LogCommand {
///     fn command_name() -> &'static str {
///         "log"
///     }
/// }
/// ```
pub trait CommandName {
    /// Command name
    fn command_name() -> &'static str;
}

/// Parse arguments from values.
///
/// # Example
///
/// ```
/// # use leafwing_terminal::{CommandArgs, FromValue, FromValueError, ValueRawOwned};
/// #
/// struct LogCommand {
///     msg: String,
/// }
///
/// impl CommandArgs for LogCommand {
///     fn from_values(values: &[ValueRawOwned]) -> Result<Self, FromValueError> {
///         let mut values = values.iter();
///         let msg = String::from_value_iter(&mut values, 0)?;
///
///         Ok(LogCommand {
///             msg
///         })
///     }
/// }
/// ```
pub trait CommandArgs: Sized {
    /// Parse arguments from values.
    fn from_values(values: &[ValueRawOwned]) -> Result<Self, FromValueError>;
}

/// Provides command usage information including description, arguments and their types.
///
/// # Example
///
/// ```
/// # use leafwing_terminal::{CommandArgInfo, CommandHelp, CommandInfo, CommandName};
/// #
/// struct LogCommand {
///     msg: String,
/// }
/// #
/// # impl CommandName for LogCommand {
/// #     fn command_name() -> &'static str {
/// #         "log"
/// #     }
/// # }
///
/// impl CommandHelp for LogCommand {
///     fn command_help() -> Option<CommandInfo> {
///         Some(CommandInfo {
///             name: "log".to_string(),
///             description: Some("Prints a message to the terminal".to_string()),
///             args: vec![
///                 CommandArgInfo {
///                     name: "msg".to_string(),
///                     ty: "string".to_string(),
///                     description: Some("message to print".to_string()),
///                     optional: false,
///                 },
///             ],
///         })
///     }
/// }
/// ```
pub trait CommandHelp: CommandName {
    /// Help for a terminal command.
    fn command_help() -> Option<CommandInfo> {
        None
    }
}

/// Command information.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandInfo {
    /// Command name
    pub name: String,
    /// Command description
    pub description: Option<String>,
    /// Command argument information
    pub args: Vec<CommandArgInfo>,
}

/// Command argument information.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandArgInfo {
    /// Argument name
    pub name: String,
    /// Argument type as string
    pub ty: String,
    /// Argument description
    pub description: Option<String>,
    /// Is argument optional
    pub optional: bool,
}

impl CommandInfo {
    /// Compine command help into usage string.
    #[allow(unused_must_use)]
    pub fn help_text(&self) -> String {
        let mut buf = "Usage:\n\n".to_string();

        write!(buf, "  > {}", self.name);
        for CommandArgInfo { name, optional, .. } in &self.args {
            write!(buf, " ");
            if *optional {
                write!(buf, "[");
            } else {
                write!(buf, "<");
            }
            write!(buf, "{name}");
            if *optional {
                write!(buf, "]");
            } else {
                write!(buf, ">");
            }
        }
        writeln!(buf);
        writeln!(buf);

        if let Some(description) = &self.description {
            let description = description.lines().fold(String::new(), |mut buf, s| {
                let spaces = s.chars().take_while(|c| *c == ' ').count();
                for _ in 0..2usize.saturating_sub(spaces) {
                    buf.push(' ');
                }
                buf.push_str(s);
                buf.push('\n');
                buf
            });
            writeln!(buf, "{description}");
        }

        let longest_arg_name = self
            .args
            .iter()
            .map(|arg| arg.name.len())
            .max()
            .unwrap_or(0);
        let longest_arg_ty = self.args.iter().map(|arg| arg.ty.len()).max().unwrap_or(0);
        for CommandArgInfo {
            name,
            ty,
            description,
            optional,
        } in &self.args
        {
            write!(
                buf,
                "    {name} {}",
                " ".repeat(longest_arg_name - name.len())
            );
            if *optional {
                write!(buf, "[");
            } else {
                write!(buf, "<");
            }
            write!(buf, "{ty}");
            if *optional {
                write!(buf, "]");
            } else {
                write!(buf, ">");
            }
            write!(buf, "{}", " ".repeat(longest_arg_ty - ty.len()));

            match description {
                Some(description) => {
                    writeln!(buf, "   - {description}");
                }
                None => {
                    writeln!(buf);
                }
            }
        }

        buf
    }
}

/// Executed parsed terminal command.
///
/// Used to capture terminal commands which implement [`CommandName`], [`CommandArgs`] & [`CommandHelp`].
/// These can be easily implemented with the [`TerminalCommand`](leafwing_terminal_derive::TerminalCommand) derive macro.
///
/// # Example
///
/// ```
/// # use leafwing_terminal::TerminalCommand;
/// #
/// /// Prints given arguments to the terminal.
/// #[derive(TerminalCommand)]
/// #[terminal_command(name = "log")]
/// struct LogCommand {
///     /// Message to print
///     msg: String,
///     /// Number of times to print message
///     num: Option<i64>,
/// }
///
/// fn log_command(mut log: TerminalCommand<LogCommand>) {
///     if let Some(LogCommand { msg, num }) = log.take() {
///         log.ok();
///     }
/// }
/// ```
pub struct TerminalCommand<'w, 's, T> {
    command: Option<T>,
    terminal_line: EventWriter<'w, 's, PrintTerminalLine>,
}

impl<'w, 's, T> TerminalCommand<'w, 's, T> {
    /// Returns Some(T) if the command was executed and arguments were valid.
    ///
    /// This method should only be called once.
    /// Consecutive calls will return None regardless if the command occured.
    pub fn take(&mut self) -> Option<T> {
        mem::take(&mut self.command)
    }

    /// Print `[ok]` in the terminal.
    pub fn ok(&mut self) {
        self.terminal_line
            .send(PrintTerminalLine::new("[ok]".to_string()));
    }

    /// Print `[failed]` in the terminal.
    pub fn failed(&mut self) {
        self.terminal_line
            .send(PrintTerminalLine::new("[failed]".to_string()));
    }

    /// Print a reply in the terminal.
    ///
    /// See [`reply!`](crate::reply) for usage with the [`format!`] syntax.
    pub fn reply(&mut self, msg: impl Into<String>) {
        self.terminal_line.send(PrintTerminalLine::new(msg.into()));
    }

    /// Print a reply in the terminal followed by `[ok]`.
    ///
    /// See [`reply_ok!`](crate::reply_ok) for usage with the [`format!`] syntax.
    pub fn reply_ok(&mut self, msg: impl Into<String>) {
        self.terminal_line.send(PrintTerminalLine::new(msg.into()));
        self.ok();
    }

    /// Print a reply in the terminal followed by `[failed]`.
    ///
    /// See [`reply_failed!`](crate::reply_failed) for usage with the [`format!`] syntax.
    pub fn reply_failed(&mut self, msg: impl Into<String>) {
        self.terminal_line.send(PrintTerminalLine::new(msg.into()));
        self.failed();
    }
}

pub struct TerminalCommandState<T> {
    #[allow(clippy::type_complexity)]
    event_reader: EventReaderState<
        (
            LocalState<(usize, PhantomData<TerminalCommandEntered>)>,
            ResState<Events<TerminalCommandEntered>>,
        ),
        TerminalCommandEntered,
    >,
    terminal_line: EventWriterState<(ResMutState<Events<PrintTerminalLine>>,), PrintTerminalLine>,
    marker: PhantomData<T>,
}

impl<'w, 's, T: Resource + CommandName + CommandArgs + CommandHelp> SystemParam
    for TerminalCommand<'w, 's, T>
{
    type Fetch = TerminalCommandState<T>;
}

unsafe impl<'w, 's, T: Resource> SystemParamState for TerminalCommandState<T> {


    fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let event_reader = EventReaderState::init(world, system_meta);
        let terminal_line = EventWriterState::init(world, system_meta);

        TerminalCommandState {
            event_reader,
            terminal_line,
            marker: PhantomData::default(),
        }
    }

}

impl<'w, 's, T: Resource + CommandName + CommandArgs + CommandHelp> SystemParamFetch<'w, 's>
    for TerminalCommandState<T>
{
    type Item = TerminalCommand<'w, 's, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        let mut event_reader =
            EventReaderState::get_param(&mut state.event_reader, system_meta, world, change_tick);
        let mut terminal_line =
            EventWriterState::get_param(&mut state.terminal_line, system_meta, world, change_tick);

        let command = event_reader
            .iter()
            .find(|cmd| cmd.command == T::command_name())
            .map(|cmd| T::from_values(&cmd.args))
            .and_then(|result| match result {
                Ok(value) => Some(value),
                Err(err) => {
                    terminal_line.send(PrintTerminalLine::new(err.to_string()));
                    match err {
                        FromValueError::UnexpectedArgType { .. }
                        | FromValueError::NotEnoughArgs
                        | FromValueError::Custom(_) => {
                            if let Some(help_text) = T::command_help() {
                                terminal_line.send(PrintTerminalLine::new(help_text.help_text()));
                            }
                        }
                        FromValueError::ValueTooLarge { .. } => {}
                    }
                    None
                }
            });

        TerminalCommand {
            command,
            terminal_line,
        }
    }
}

/// Parsed raw terminal command into `command` and `args`.
#[derive(Clone, Debug, PartialEq)]
pub struct TerminalCommandEntered {
    /// Command name
    pub command: String,
    /// Raw parsed arguments
    pub args: Vec<ValueRawOwned>,
}

/// Events to print to the terminal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrintTerminalLine {
    /// Terminal line
    pub line: String,
}

impl PrintTerminalLine {
    /// Creates a new terminal line to print.
    pub const fn new(line: String) -> Self {
        Self { line }
    }
}

/// Terminal configuration
#[derive(Clone)]
pub struct TerminalConfiguration {
    /// Registered keys for toggling the terminal
    /// Left position
    pub left_pos: f32,
    /// Top position
    pub top_pos: f32,
    /// Terminal height
    pub height: f32,
    /// Terminal width
    pub width: f32,
    /// Registered terminal commands
    pub commands: BTreeMap<&'static str, Option<CommandInfo>>,
    /// Number of commands to store in history
    pub history_size: usize,
}

impl Default for TerminalConfiguration {
    fn default() -> Self {
        Self {
            left_pos: 200.0,
            top_pos: 100.0,
            height: 400.0,
            width: 800.0,
            commands: BTreeMap::new(),
            history_size: 20,
        }
    }
}

/// Add a terminal commands to Bevy app.
pub trait AddTerminalCommand {
    /// Add a terminal command with a given system.
    ///
    /// This registers the terminal command so it will print with the built-in `help` terminal command.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use leafwing_terminal::{AddTerminalCommand, TerminalCommand};
    /// #
    /// App::new()
    ///     .add_terminal_command::<LogCommand, _, _>(log_command);
    /// #
    /// # /// Prints given arguments to the terminal.
    /// # #[derive(TerminalCommand)]
    /// # #[terminal_command(name = "log")]
    /// # struct LogCommand;
    /// #
    /// # fn log_command(mut log: TerminalCommand<LogCommand>) {}
    /// ```
    fn add_terminal_command<T: CommandName + CommandHelp, Sys, Params>(
        &mut self,
        system: Sys,
    ) -> &mut Self
    where
        Sys: IntoSystemDescriptor<Params>;
}

impl AddTerminalCommand for App {
    fn add_terminal_command<T: CommandName + CommandHelp, Sys, Params>(
        &mut self,
        system: Sys,
    ) -> &mut Self
    where
        Sys: IntoSystemDescriptor<Params>,
    {
        let sys = move |mut config: ResMut<TerminalConfiguration>| {
            let name = T::command_name();
            if config.commands.contains_key(name) {
                warn!(
                    "terminal command '{}' already registered and was overwritten",
                    name
                );
            }
            config.commands.insert(name, T::command_help());
        };

        self.add_startup_system(sys).add_system(system)
    }
}

pub(crate) struct TerminalState {
    pub(crate) buf: String,
    pub(crate) scrollback: Vec<String>,
    pub(crate) history: VecDeque<String>,
    pub(crate) history_index: usize,
}

impl Default for TerminalState {
    fn default() -> Self {
        TerminalState {
            buf: String::default(),
            scrollback: Vec::new(),
            history: VecDeque::from([String::new()]),
            history_index: 0,
        }
    }
}

pub(crate) fn receive_terminal_line(
    mut terminal_state: ResMut<TerminalState>,
    mut events: EventReader<PrintTerminalLine>,
) {
    for event in events.iter() {
        let event: &PrintTerminalLine = event;
        terminal_state.scrollback.push(event.line.clone());
    }
}
