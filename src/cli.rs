use clap::{Parser, Subcommand, ValueEnum};
use honk_engine::PokeAction;

#[derive(Debug, Parser)]
#[command(name = "honk300", version, about = "A desktop goose for your screen")]
pub struct Cli {
    /// Start the goose muted.
    #[arg(long, alias = "silent", global = true)]
    pub no_sound: bool,

    /// Disable cursor-stealing behavior for this run.
    #[arg(long, global = true)]
    pub no_mouse_steal: bool,

    /// Disable foreign-window ride behavior for this run.
    #[arg(long, global = true)]
    pub no_window_ride: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Start the goose. This is also the default when no command is provided.
    Start,
    /// Stop the running goose through local IPC.
    Stop,
    /// Ask the running goose to reload runtime options.
    Reload,
    /// Poke the running goose into a specific action.
    Do {
        #[arg(value_enum)]
        action: CliPokeAction,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliPokeAction {
    Honk,
    Wander,
    Mud,
    Meme,
    Note,
    Nab,
}

impl CliPokeAction {
    pub fn into_engine(self) -> PokeAction {
        match self {
            Self::Honk => PokeAction::Honk,
            Self::Wander => PokeAction::Wander,
            Self::Mud => PokeAction::Mud,
            Self::Meme => PokeAction::Meme,
            Self::Note => PokeAction::Note,
            Self::Nab => PokeAction::Nab,
        }
    }
}

impl Cli {
    pub fn is_start(&self) -> bool {
        self.command.is_none() || self.command == Some(Command::Start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn empty_command_defaults_to_start() {
        let cli = Cli::parse_from(["honk300"]);
        assert!(cli.is_start());
    }

    #[test]
    fn parses_explicit_start() {
        let cli = Cli::parse_from(["honk300", "start"]);
        assert!(cli.is_start());
    }

    #[test]
    fn parses_stop_reload_and_do() {
        let stop = Cli::parse_from(["honk300", "stop"]);
        assert_eq!(stop.command, Some(Command::Stop));

        let reload = Cli::parse_from(["honk300", "reload"]);
        assert_eq!(reload.command, Some(Command::Reload));

        let do_note = Cli::parse_from(["honk300", "do", "note"]);
        assert_eq!(
            do_note.command,
            Some(Command::Do {
                action: CliPokeAction::Note
            })
        );
    }

    #[test]
    fn maps_cli_actions_to_engine_actions() {
        assert_eq!(CliPokeAction::Meme.into_engine(), PokeAction::Meme);
        assert_eq!(CliPokeAction::Nab.into_engine(), PokeAction::Nab);
    }
}
