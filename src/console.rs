use std::collections::{BTreeMap, VecDeque};
use std::marker::PhantomData;
use std::{fmt::Write, mem};

use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::{
    app::{EventReaderState, EventWriterState, Events},
    ecs::system::{
        LocalState, ResMutState, ResState, Resource, SystemMeta, SystemParam, SystemParamFetch,
        SystemParamState,
    },
    prelude::*,
};
use leafwing_terminal_parser::ValueRawOwned;

use crate::FromValueError;

/// Console command name.
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
///             description: Some("Prints a message to the console".to_string()),
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
    /// Help for a console command.
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

/// Executed parsed console command.
///
/// Used to capture console commands which implement [`CommandName`], [`CommandArgs`] & [`CommandHelp`].
/// These can be easily implemented with the [`ConsoleCommand`](leafwing_terminal_derive::ConsoleCommand) derive macro.
///
/// # Example
///
/// ```
/// # use leafwing_terminal::ConsoleCommand;
/// #
/// /// Prints given arguments to the console.
/// #[derive(ConsoleCommand)]
/// #[console_command(name = "log")]
/// struct LogCommand {
///     /// Message to print
///     msg: String,
///     /// Number of times to print message
///     num: Option<i64>,
/// }
///
/// fn log_command(mut log: ConsoleCommand<LogCommand>) {
///     if let Some(LogCommand { msg, num }) = log.take() {
///         log.ok();
///     }
/// }
/// ```
pub struct ConsoleCommand<'w, 's, T> {
    command: Option<T>,
    console_line: EventWriter<'w, 's, PrintConsoleLine>,
}

impl<'w, 's, T> ConsoleCommand<'w, 's, T> {
    /// Returns Some(T) if the command was executed and arguments were valid.
    ///
    /// This method should only be called once.
    /// Consecutive calls will return None regardless if the command occured.
    pub fn take(&mut self) -> Option<T> {
        mem::take(&mut self.command)
    }

    /// Print `[ok]` in the console.
    pub fn ok(&mut self) {
        self.console_line
            .send(PrintConsoleLine::new("[ok]".to_string()));
    }

    /// Print `[failed]` in the console.
    pub fn failed(&mut self) {
        self.console_line
            .send(PrintConsoleLine::new("[failed]".to_string()));
    }

    /// Print a reply in the console.
    ///
    /// See [`reply!`](crate::reply) for usage with the [`format!`] syntax.
    pub fn reply(&mut self, msg: impl Into<String>) {
        self.console_line.send(PrintConsoleLine::new(msg.into()));
    }

    /// Print a reply in the console followed by `[ok]`.
    ///
    /// See [`reply_ok!`](crate::reply_ok) for usage with the [`format!`] syntax.
    pub fn reply_ok(&mut self, msg: impl Into<String>) {
        self.console_line.send(PrintConsoleLine::new(msg.into()));
        self.ok();
    }

    /// Print a reply in the console followed by `[failed]`.
    ///
    /// See [`reply_failed!`](crate::reply_failed) for usage with the [`format!`] syntax.
    pub fn reply_failed(&mut self, msg: impl Into<String>) {
        self.console_line.send(PrintConsoleLine::new(msg.into()));
        self.failed();
    }
}

pub struct ConsoleCommandState<T> {
    #[allow(clippy::type_complexity)]
    event_reader: EventReaderState<
        (
            LocalState<(usize, PhantomData<ConsoleCommandEntered>)>,
            ResState<Events<ConsoleCommandEntered>>,
        ),
        ConsoleCommandEntered,
    >,
    console_line: EventWriterState<(ResMutState<Events<PrintConsoleLine>>,), PrintConsoleLine>,
    marker: PhantomData<T>,
}

impl<'w, 's, T: Resource + CommandName + CommandArgs + CommandHelp> SystemParam
    for ConsoleCommand<'w, 's, T>
{
    type Fetch = ConsoleCommandState<T>;
}

unsafe impl<'w, 's, T: Resource> SystemParamState for ConsoleCommandState<T> {
    type Config = ();

    fn init(world: &mut World, system_meta: &mut SystemMeta, _config: Self::Config) -> Self {
        let event_reader = EventReaderState::init(world, system_meta, (None, ()));
        let console_line = EventWriterState::init(world, system_meta, ((),));

        ConsoleCommandState {
            event_reader,
            console_line,
            marker: PhantomData::default(),
        }
    }

    fn default_config() {}
}

impl<'w, 's, T: Resource + CommandName + CommandArgs + CommandHelp> SystemParamFetch<'w, 's>
    for ConsoleCommandState<T>
{
    type Item = ConsoleCommand<'w, 's, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        let mut event_reader =
            EventReaderState::get_param(&mut state.event_reader, system_meta, world, change_tick);
        let mut console_line =
            EventWriterState::get_param(&mut state.console_line, system_meta, world, change_tick);

        let command = event_reader
            .iter()
            .find(|cmd| cmd.command == T::command_name())
            .map(|cmd| T::from_values(&cmd.args))
            .and_then(|result| match result {
                Ok(value) => Some(value),
                Err(err) => {
                    console_line.send(PrintConsoleLine::new(err.to_string()));
                    match err {
                        FromValueError::UnexpectedArgType { .. }
                        | FromValueError::NotEnoughArgs
                        | FromValueError::Custom(_) => {
                            if let Some(help_text) = T::command_help() {
                                console_line.send(PrintConsoleLine::new(help_text.help_text()));
                            }
                        }
                        FromValueError::ValueTooLarge { .. } => {}
                    }
                    None
                }
            });

        ConsoleCommand {
            command,
            console_line,
        }
    }
}

/// Parsed raw console command into `command` and `args`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleCommandEntered {
    /// Command name
    pub command: String,
    /// Raw parsed arguments
    pub args: Vec<ValueRawOwned>,
}

/// Events to print to the console.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrintConsoleLine {
    /// Console line
    pub line: String,
}

impl PrintConsoleLine {
    /// Creates a new console line to print.
    pub const fn new(line: String) -> Self {
        Self { line }
    }
}

/// Console configuration
#[derive(Clone)]
pub struct ConsoleConfiguration {
    /// Registered keys for toggling the console
    /// Left position
    pub left_pos: f32,
    /// Top position
    pub top_pos: f32,
    /// Console height
    pub height: f32,
    /// Console width
    pub width: f32,
    /// Registered console commands
    pub commands: BTreeMap<&'static str, Option<CommandInfo>>,
    /// Number of commands to store in history
    pub history_size: usize,
}

impl Default for ConsoleConfiguration {
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

/// Add a console commands to Bevy app.
pub trait AddConsoleCommand {
    /// Add a console command with a given system.
    ///
    /// This registers the console command so it will print with the built-in `help` console command.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use leafwing_terminal::{AddConsoleCommand, ConsoleCommand};
    /// #
    /// App::new()
    ///     .add_console_command::<LogCommand, _, _>(log_command);
    /// #
    /// # /// Prints given arguments to the console.
    /// # #[derive(ConsoleCommand)]
    /// # #[console_command(name = "log")]
    /// # struct LogCommand;
    /// #
    /// # fn log_command(mut log: ConsoleCommand<LogCommand>) {}
    /// ```
    fn add_console_command<T: CommandName + CommandHelp, Sys, Params>(
        &mut self,
        system: Sys,
    ) -> &mut Self
    where
        Sys: IntoSystemDescriptor<Params>;
}

impl AddConsoleCommand for App {
    fn add_console_command<T: CommandName + CommandHelp, Sys, Params>(
        &mut self,
        system: Sys,
    ) -> &mut Self
    where
        Sys: IntoSystemDescriptor<Params>,
    {
        let sys = move |mut config: ResMut<ConsoleConfiguration>| {
            let name = T::command_name();
            if config.commands.contains_key(name) {
                warn!(
                    "console command '{}' already registered and was overwritten",
                    name
                );
            }
            config.commands.insert(name, T::command_help());
        };

        self.add_startup_system(sys).add_system(system)
    }
}

pub(crate) struct ConsoleState {
    pub(crate) buf: String,
    pub(crate) scrollback: Vec<String>,
    pub(crate) history: VecDeque<String>,
    pub(crate) history_index: usize,
}

impl Default for ConsoleState {
    fn default() -> Self {
        ConsoleState {
            buf: String::default(),
            scrollback: Vec::new(),
            history: VecDeque::from([String::new()]),
            history_index: 0,
        }
    }
}

pub(crate) fn receive_console_line(
    mut console_state: ResMut<ConsoleState>,
    mut events: EventReader<PrintConsoleLine>,
) {
    for event in events.iter() {
        let event: &PrintConsoleLine = event;
        console_state.scrollback.push(event.line.clone());
    }
}
