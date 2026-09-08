use super::*;

fn translated_state(role: Role, disabled: bool, read_only: bool) -> StateSet {
    let mut node = accesskit::Node::new(role);
    if disabled {
        node.set_disabled();
    }
    if read_only {
        node.set_read_only();
    }
    let id = LocalNodeId(1);
    let tree = Tree::new(
        accesskit::TreeUpdate {
            tree_id: TreeId::ROOT,
            nodes: vec![(id, node)],
            tree: Some(accesskit::Tree::new(id)),
            focus: id,
        },
        true,
    );
    NodeWrapper(&tree.state().root()).state(true)
}

#[test]
fn disabled_buttons_and_switches_are_neither_enabled_nor_sensitive() {
    for role in [
        Role::Button,
        Role::DefaultButton,
        Role::Switch,
        Role::CheckBox,
    ] {
        let states = translated_state(role, true, false);
        assert!(
            !states.contains(State::Enabled),
            "disabled {role:?} is enabled"
        );
        assert!(
            !states.contains(State::Sensitive),
            "disabled {role:?} is sensitive"
        );
    }
}

#[test]
fn available_buttons_remain_enabled_and_sensitive() {
    for role in [
        Role::Button,
        Role::DefaultButton,
        Role::Switch,
        Role::CheckBox,
    ] {
        let states = translated_state(role, false, false);
        assert!(
            states.contains(State::Enabled),
            "available {role:?} is disabled"
        );
        assert!(
            states.contains(State::Sensitive),
            "available {role:?} is insensitive"
        );
    }
}

#[test]
fn read_only_text_state_is_preserved() {
    let states = translated_state(Role::TextInput, false, true);
    assert!(states.contains(State::ReadOnly));
    assert!(!states.contains(State::Enabled));
    assert!(!states.contains(State::Sensitive));
}
