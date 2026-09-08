//! A bounded adapter from the Native SDK widget snapshot to the real OS tree.
//! The owning UI thread creates, publishes, polls, and drops the bridge. OS
//! callbacks only read the retained tree or queue actions; they never enter Zig.

use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Rect, Role,
    Toggled, Tree, TreeUpdate,
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::c_void,
    sync::{Arc, Mutex},
};

const ROOT: NodeId = NodeId(u64::MAX);
const MAX_NODES: usize = 128;
const MAX_JSON: usize = 256 * 1024;
const MAX_ACTIONS: usize = 32;
const MAX_TEXT: usize = 4096;

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
struct Actions {
    focus: bool,
    press: bool,
    toggle: bool,
    increment: bool,
    decrement: bool,
    set_text: bool,
    select: bool,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Deserialize)]
struct Widget {
    id: u64,
    parent_id: Option<u64>,
    role: String,
    label: String,
    text_value: String,
    placeholder: String,
    value: Option<f64>,
    bounds: Bounds,
    enabled: bool,
    focused: bool,
    selected: bool,
    expanded: Option<bool>,
    required: bool,
    read_only: bool,
    invalid: bool,
    actions: Actions,
}

#[derive(Deserialize)]
struct Snapshot {
    nodes: Vec<Widget>,
}

#[derive(Default)]
struct Shared {
    tree: Option<TreeUpdate>,
    actions: HashMap<u64, Actions>,
    queue: VecDeque<ActionRequest>,
}

#[derive(Clone)]
struct Handler(Arc<Mutex<Shared>>);

impl ActivationHandler for Handler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.0.lock().ok()?.tree.clone()
    }
}

impl ActionHandler for Handler {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut state) = self.0.lock() {
            if state.queue.len() < MAX_ACTIONS && allowed(&state, &request) {
                state.queue.push_back(request);
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl accesskit::DeactivationHandler for Handler {
    fn deactivate_accessibility(&mut self) {}
}

fn allowed(state: &Shared, request: &ActionRequest) -> bool {
    if request.target_tree != accesskit::TreeId::ROOT {
        return false;
    }
    let Some(actions) = state.actions.get(&request.target_node.0) else {
        return false;
    };
    match request.action {
        Action::Focus => actions.focus,
        Action::Click => actions.press || actions.toggle || actions.select,
        Action::Increment => actions.increment,
        Action::Decrement => actions.decrement,
        Action::SetValue => {
            actions.set_text
                && matches!(&request.data,
            Some(ActionData::Value(text)) if text.len() <= MAX_TEXT && !text.contains('\0'))
        }
        _ => false,
    }
}

fn role(value: &str) -> Role {
    match value {
        "group" => Role::GenericContainer,
        "text" => Role::Label,
        "image" => Role::Image,
        "button" => Role::Button,
        "textbox" => Role::TextInput,
        "tooltip" => Role::Tooltip,
        "dialog" => Role::Dialog,
        "menu" => Role::Menu,
        "menuitem" => Role::MenuItem,
        "list" => Role::List,
        "listitem" => Role::ListItem,
        "row" => Role::Row,
        "grid" => Role::Grid,
        "gridcell" => Role::Cell,
        "tab" => Role::Tab,
        "checkbox" => Role::CheckBox,
        "switch_control" => Role::Switch,
        "slider" => Role::Slider,
        "progressbar" => Role::ProgressIndicator,
        "radio" => Role::RadioButton,
        _ => Role::GenericContainer,
    }
}

fn convert(mut snapshot: Snapshot, scale: f64) -> Option<(TreeUpdate, HashMap<u64, Actions>)> {
    if snapshot.nodes.len() > MAX_NODES || !scale.is_finite() || !(0.5..=8.0).contains(&scale) {
        return None;
    }
    let ids: HashSet<u64> = snapshot.nodes.iter().map(|node| node.id).collect();
    if ids.len() != snapshot.nodes.len()
        || ids.iter().any(|id| *id >= ROOT.0 - MAX_NODES as u64)
        || snapshot
            .nodes
            .iter()
            .any(|node| node.text_value.len() > MAX_TEXT || node.label.len() > MAX_TEXT)
    {
        return None;
    }
    let parents: HashMap<_, _> = snapshot.nodes.iter().map(|n| (n.id, n.parent_id)).collect();
    for widget in &snapshot.nodes {
        let mut parent = widget.parent_id;
        for depth in 0..=MAX_NODES {
            match parent {
                Some(id) if id == widget.id || depth == MAX_NODES => return None,
                Some(id) => parent = *parents.get(&id)?,
                None => break,
            }
        }
    }
    // The settings editor uses modal dialogs. The toolkit still publishes the
    // obscured page; keep those controls out of native assistive navigation.
    if let Some(dialog) = snapshot
        .nodes
        .iter()
        .rev()
        .find(|n| n.role == "dialog")
        .map(|n| n.id)
    {
        snapshot.nodes.retain(|node| {
            let mut cursor = Some(node.id);
            while let Some(id) = cursor {
                if id == dialog {
                    return true;
                }
                cursor = parents[&id];
            }
            false
        });
        snapshot
            .nodes
            .iter_mut()
            .find(|n| n.id == dialog)?
            .parent_id = None;
    }
    let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for widget in &snapshot.nodes {
        children
            .entry(widget.parent_id.map(NodeId).unwrap_or(ROOT))
            .or_default()
            .push(NodeId(widget.id));
    }
    let mut root = Node::new(Role::Window);
    root.set_label("Honk300 settings");
    root.set_children(children.remove(&ROOT).unwrap_or_default());
    let mut nodes = vec![(ROOT, root)];
    let mut actions = HashMap::new();
    let mut focus = ROOT;
    for (index, widget) in snapshot.nodes.into_iter().enumerate() {
        let b = widget.bounds;
        if [b.x, b.y, b.width, b.height].iter().any(|v| !v.is_finite())
            || b.width < 0.0
            || b.height < 0.0
        {
            return None;
        }
        let mut node = Node::new(role(&widget.role));
        if widget.role == "text" {
            // AccessKit exposes static text through its value, including the
            // Windows UIA Name property. Labels alone are ignored for this role.
            node.set_value(if widget.text_value.is_empty() {
                widget.label.clone()
            } else {
                widget.text_value.clone()
            });
        } else if !widget.label.is_empty() {
            node.set_label(widget.label);
        }
        node.set_bounds(Rect::new(
            b.x * scale,
            b.y * scale,
            (b.x + b.width) * scale,
            (b.y + b.height) * scale,
        ));
        node.set_children(children.remove(&NodeId(widget.id)).unwrap_or_default());
        if !widget.text_value.is_empty() {
            node.set_value(widget.text_value);
        }
        if !widget.placeholder.is_empty() {
            node.set_placeholder(widget.placeholder);
        }
        if let Some(value) = widget.value {
            node.set_numeric_value(value);
        }
        if matches!(
            widget.role.as_str(),
            "checkbox" | "switch_control" | "radio"
        ) {
            node.set_toggled(if widget.selected {
                Toggled::True
            } else {
                Toggled::False
            });
        } else if matches!(widget.role.as_str(), "tab" | "listitem") {
            node.set_selected(widget.selected);
        }
        if let Some(expanded) = widget.expanded {
            node.set_expanded(expanded);
        }
        if widget.required {
            node.set_required();
        }
        if widget.read_only {
            node.set_read_only();
        }
        if widget.invalid {
            node.set_invalid(accesskit::Invalid::True);
        }
        if !widget.enabled {
            node.set_disabled();
        }
        if widget.focused {
            focus = NodeId(widget.id);
        }
        if widget.enabled {
            let mut a = widget.actions;
            a.set_text &= !widget.read_only;
            if a.focus {
                node.add_action(Action::Focus);
            }
            if a.press || a.toggle || a.select {
                node.add_action(Action::Click);
            }
            if a.increment {
                node.add_action(Action::Increment);
            }
            if a.decrement {
                node.add_action(Action::Decrement);
            }
            if a.set_text && !widget.read_only {
                node.add_action(Action::SetValue);
            }
            actions.insert(widget.id, a);
        }
        // AT-SPI reads field contents through Text, which requires a TextRun.
        // Retain UTF-8 character boundaries without inventing glyph geometry or
        // cursor selection that the toolkit has not published.
        if matches!(widget.role.as_str(), "textbox" | "text") {
            let text = node.value().unwrap_or("").to_owned();
            let text_id = NodeId(ROOT.0 - 1 - index as u64);
            let mut run = Node::new(Role::TextRun);
            run.set_character_lengths(text.chars().map(|c| c.len_utf8() as u8).collect::<Vec<_>>());
            run.set_value(text);
            let mut text_children = node.children().to_vec();
            text_children.push(text_id);
            node.set_children(text_children);
            nodes.push((NodeId(widget.id), node));
            nodes.push((text_id, run));
        } else {
            nodes.push((NodeId(widget.id), node));
        }
    }
    let mut tree = Tree::new(ROOT);
    tree.toolkit_name = Some("Native SDK / AccessKit".into());
    Some((
        TreeUpdate {
            nodes,
            tree: Some(tree),
            focus,
            tree_id: accesskit::TreeId::ROOT,
        },
        actions,
    ))
}

pub struct Bridge {
    shared: Arc<Mutex<Shared>>,
    #[cfg(windows)]
    adapter: accesskit_windows::Adapter,
    #[cfg(target_os = "linux")]
    adapter: accesskit_unix::Adapter,
}

#[repr(C)]
pub struct NativeAction {
    pub id: u64,
    pub action: i32,
    pub text_len: usize,
    pub text: [u8; MAX_TEXT],
}

/// # Safety
/// Windows `window` must be a live HWND owned by this thread. Create outside WM_GETOBJECT.
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_create(window: *mut c_void) -> *mut Bridge {
    let shared = Arc::new(Mutex::new(Shared::default()));
    #[cfg(windows)]
    let adapter = accesskit_windows::Adapter::new(
        accesskit_windows::HWND(window),
        false,
        Handler(shared.clone()),
    );
    #[cfg(target_os = "linux")]
    let adapter = {
        let _ = window;
        accesskit_unix::Adapter::new(
            Handler(shared.clone()),
            Handler(shared.clone()),
            Handler(shared.clone()),
        )
    };
    #[cfg(not(any(windows, target_os = "linux")))]
    let _ = window;
    Box::into_raw(Box::new(Bridge {
        shared,
        #[cfg(any(windows, target_os = "linux"))]
        adapter,
    }))
}

/// # Safety
/// `bridge` is a live handle, used only on the owner thread. Free exactly once before HWND destruction.
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_destroy(bridge: *mut Bridge) {
    if !bridge.is_null() {
        drop(Box::from_raw(bridge));
    }
}

/// # Safety
/// `bridge` is live and exclusively accessed; `json` points to `len` readable bytes for this call.
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_publish(
    bridge: *mut Bridge,
    json: *const u8,
    len: usize,
    scale: f64,
) -> i32 {
    let Some(bridge) = bridge.as_mut() else {
        return 0;
    };
    if json.is_null() || len > MAX_JSON {
        return 0;
    }
    let Ok(snapshot) = serde_json::from_slice::<Snapshot>(std::slice::from_raw_parts(json, len))
    else {
        return 0;
    };
    let Some((tree, actions)) = convert(snapshot, scale) else {
        return 0;
    };
    let Ok(mut shared) = bridge.shared.lock() else {
        return 0;
    };
    shared.tree = Some(tree.clone());
    shared.actions = actions;
    drop(shared);
    #[cfg(windows)]
    if let Some(events) = bridge.adapter.update_if_active(|| tree) {
        events.raise();
    }
    #[cfg(target_os = "linux")]
    bridge.adapter.update_if_active(|| tree);
    1
}

/// # Safety
/// The live bridge belongs to this UI thread. `output` is writable for one NativeAction.
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_poll(bridge: *mut Bridge, output: *mut NativeAction) -> i32 {
    let (Some(bridge), Some(output)) = (bridge.as_mut(), output.as_mut()) else {
        return 0;
    };
    let Ok(mut state) = bridge.shared.lock() else {
        return 0;
    };
    while let Some(request) = state.queue.pop_front() {
        // Recheck against the current tree: queued actions cannot operate stale or disabled controls.
        if !allowed(&state, &request) {
            continue;
        }
        let actions = &state.actions[&request.target_node.0];
        let kind = match request.action {
            Action::Focus => 0,
            Action::Click if actions.toggle => 2,
            Action::Click if actions.select && !actions.press => 7,
            Action::Click => 1,
            Action::Increment => 3,
            Action::Decrement => 4,
            Action::SetValue => 5,
            _ => continue,
        };
        *output = NativeAction {
            id: request.target_node.0,
            action: kind,
            text_len: 0,
            text: [0; MAX_TEXT],
        };
        if let Some(ActionData::Value(text)) = request.data {
            output.text_len = text.len();
            output.text[..text.len()].copy_from_slice(text.as_bytes());
        }
        return 1;
    }
    0
}

/// # Safety
/// `bridge` is a live owner-thread handle and `result` points to a writable isize.
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_getobject(
    bridge: *mut Bridge,
    wparam: usize,
    lparam: isize,
    result: *mut isize,
) -> i32 {
    let (Some(bridge), Some(result)) = (bridge.as_mut(), result.as_mut()) else {
        return 0;
    };
    let mut handler = Handler(bridge.shared.clone());
    match bridge.adapter.handle_wm_getobject(
        accesskit_windows::WPARAM(wparam),
        accesskit_windows::LPARAM(lparam),
        &mut handler,
    ) {
        Some(value) => {
            let value: accesskit_windows::LRESULT = value.into();
            *result = value.0;
            1
        }
        None => 0,
    }
}

/// # Safety
/// `bridge` is a live owner-thread handle.
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_focus(bridge: *mut Bridge, focused: i32) {
    let Some(bridge) = bridge.as_mut() else {
        return;
    };
    #[cfg(windows)]
    if let Some(events) = bridge.adapter.update_window_focus_state(focused != 0) {
        events.raise();
    }
    #[cfg(target_os = "linux")]
    bridge.adapter.update_window_focus_state(focused != 0);
    #[cfg(not(any(windows, target_os = "linux")))]
    let _ = (bridge, focused);
}

/// # Safety
/// `bridge` is live on the owner thread. Bounds are physical X11 screen coordinates.
#[cfg(target_os = "linux")]
#[no_mangle]
pub unsafe extern "C" fn honk_a11y_bounds(
    bridge: *mut Bridge,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) {
    let Some(bridge) = bridge.as_mut() else {
        return;
    };
    if [x, y, width, height].iter().all(|v| v.is_finite()) && width >= 0.0 && height >= 0.0 {
        let bounds = Rect::new(x, y, x + width, y + height);
        bridge.adapter.set_root_window_bounds(bounds, bounds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn widget(id: u64) -> Widget {
        Widget {
            id,
            parent_id: None,
            role: "checkbox".into(),
            label: "Reduced motion".into(),
            text_value: String::new(),
            placeholder: String::new(),
            value: None,
            bounds: Bounds {
                x: 12.0,
                y: 20.0,
                width: 120.0,
                height: 24.0,
            },
            enabled: true,
            focused: true,
            selected: true,
            expanded: None,
            required: false,
            read_only: false,
            invalid: false,
            actions: Actions {
                focus: true,
                toggle: true,
                ..Default::default()
            },
        }
    }
    #[test]
    fn real_roles_state_focus_and_scaled_bounds_survive_conversion() {
        let (tree, _) = convert(
            Snapshot {
                nodes: vec![widget(7)],
            },
            1.5,
        )
        .unwrap();
        let node = &tree.nodes[1].1;
        assert_eq!(node.role(), Role::CheckBox);
        assert_eq!(node.toggled(), Some(Toggled::True));
        assert_eq!(node.bounds(), Some(Rect::new(18.0, 30.0, 198.0, 66.0)));
        assert_eq!(tree.focus, NodeId(7));
        assert!(node.supports_action(Action::Click));
    }
    #[test]
    fn malformed_or_cyclic_tree_never_replaces_accessibility_state() {
        let mut child = widget(7);
        child.parent_id = Some(7);
        assert!(convert(Snapshot { nodes: vec![child] }, 1.0).is_none());
        assert!(convert(
            Snapshot {
                nodes: vec![widget(7), widget(7)]
            },
            1.0
        )
        .is_none());
        assert!(convert(
            Snapshot {
                nodes: vec![widget(7)]
            },
            f64::NAN
        )
        .is_none());
    }
    #[test]
    fn modal_navigation_excludes_background_and_static_text_has_a_value() {
        let mut dialog = widget(8);
        dialog.role = "dialog".into();
        dialog.focused = false;
        let mut text = widget(9);
        text.role = "text".into();
        text.parent_id = Some(8);
        text.actions = Actions::default();
        text.focused = false;
        let (tree, actions) = convert(
            Snapshot {
                nodes: vec![widget(7), dialog, text],
            },
            1.0,
        )
        .unwrap();
        assert_eq!(tree.nodes.len(), 4);
        assert_eq!(tree.nodes[0].1.children(), &[NodeId(8)]);
        assert_eq!(tree.nodes[2].1.value(), Some("Reduced motion"));
        assert!(!actions.contains_key(&7));
    }
    #[test]
    fn native_consumer_reads_unicode_and_empty_editor_contents() {
        for value in ["25", "Caf\u{e9} \u{1f986}", ""] {
            let mut field = widget(7);
            field.role = "textbox".into();
            field.text_value = value.into();
            let (update, _) = convert(Snapshot { nodes: vec![field] }, 1.0).unwrap();
            let tree = accesskit_consumer::Tree::new(update, true);
            let node = tree.state().focus().unwrap();
            assert!(node.supports_text_ranges());
            assert_eq!(node.document_range().text(), value);
        }
    }
    #[test]
    fn stale_and_disabled_actions_are_rejected_and_queue_is_bounded() {
        let (_, actions) = convert(
            Snapshot {
                nodes: vec![widget(7)],
            },
            1.0,
        )
        .unwrap();
        let shared = Arc::new(Mutex::new(Shared {
            actions,
            ..Default::default()
        }));
        let mut handler = Handler(shared.clone());
        let request = ActionRequest {
            action: Action::Click,
            target_tree: accesskit::TreeId::ROOT,
            target_node: NodeId(7),
            data: None,
        };
        for _ in 0..100 {
            handler.do_action(request.clone());
        }
        assert_eq!(shared.lock().unwrap().queue.len(), MAX_ACTIONS);
        shared.lock().unwrap().actions.clear();
        assert!(!allowed(&shared.lock().unwrap(), &request));
        let mut disabled = widget(7);
        disabled.enabled = false;
        let (_, actions) = convert(
            Snapshot {
                nodes: vec![disabled],
            },
            1.0,
        )
        .unwrap();
        assert!(actions.is_empty());
    }
}
