use super::state::{Category, CommandResult};
use honk_engine::PokeAction;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Quit,
    NextCategory,
    PrevCategory,
    SelectCategory(Category),
    MoveDown,
    MoveUp,
    ScrollStatus(i8),
    Toggle,
    Adjust(i8),
    Save,
    Reload,
    Status,
    Stop,
    Start,
    Poke(PokeAction),
    CommandResult(Box<CommandResult>),
}
