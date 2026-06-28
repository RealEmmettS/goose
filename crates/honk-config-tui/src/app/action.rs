use super::state::{Category, CommandResult};
use honk_engine::PokeAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    NextCategory,
    PrevCategory,
    SelectCategory(Category),
    MoveDown,
    MoveUp,
    Toggle,
    Adjust(i8),
    Save,
    Reload,
    Stop,
    Start,
    Poke(PokeAction),
    CommandResult(CommandResult),
}
