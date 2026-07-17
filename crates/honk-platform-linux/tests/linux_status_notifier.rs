#![cfg(target_os = "linux")]

use honk_control::ControlSurfaceCommand;
use honk_platform_linux::StatusTray;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zbus::blocking::{connection, Connection, Proxy};
use zbus::zvariant::OwnedValue;

const WATCHER_NAME: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const SNI_PATH: &str = "/StatusNotifierItem";
const SNI_INTERFACE: &str = "org.kde.StatusNotifierItem";
const MENU_PATH: &str = "/MenuBar";
const MENU_INTERFACE: &str = "com.canonical.dbusmenu";
const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Default)]
struct WatcherState {
    registered_items: Vec<String>,
    host_registered: bool,
}

struct MockWatcher {
    state: Arc<Mutex<WatcherState>>,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl MockWatcher {
    async fn register_status_notifier_item(&self, service: &str) {
        self.state
            .lock()
            .expect("watcher state lock")
            .registered_items
            .push(service.to_owned());
    }

    async fn register_status_notifier_host(&self, _service: &str) {}

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("watcher state lock")
            .registered_items
            .clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        self.state
            .lock()
            .expect("watcher state lock")
            .host_registered
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
}

struct Watcher {
    connection: Connection,
    state: Arc<Mutex<WatcherState>>,
}

impl Watcher {
    fn start(host_registered: bool) -> Self {
        let state = Arc::new(Mutex::new(WatcherState {
            registered_items: Vec::new(),
            host_registered,
        }));
        let connection = connection::Builder::session()
            .expect("session bus builder")
            .method_timeout(TIMEOUT)
            .serve_at(
                WATCHER_PATH,
                MockWatcher {
                    state: state.clone(),
                },
            )
            .expect("watcher object")
            .name(WATCHER_NAME)
            .expect("watcher bus name")
            .build()
            .expect("watcher connection");
        Self { connection, state }
    }

    fn wait_for_registration(&self, count: usize) -> String {
        wait_until("StatusNotifierItem registration", || {
            self.state
                .lock()
                .expect("watcher state lock")
                .registered_items
                .len()
                >= count
        });
        self.state
            .lock()
            .expect("watcher state lock")
            .registered_items[count - 1]
            .clone()
    }

    fn close(self) {
        self.connection.close().expect("watcher close");
    }
}

type LayoutTuple = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);

fn find_menu_item(layout: LayoutTuple, expected_label: &str) -> Option<i32> {
    let (id, properties, children) = layout;
    let label: Option<String> = properties
        .get("label")
        .and_then(|value| value.clone().try_into().ok());
    if label.as_deref() == Some(expected_label) {
        return Some(id);
    }
    children.into_iter().find_map(|child| {
        let child: LayoutTuple = child.try_into().expect("valid dbusmenu child layout");
        find_menu_item(child, expected_label)
    })
}

fn wait_until(description: &str, mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + TIMEOUT;
    while Instant::now() < deadline {
        if condition() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for {description}");
}

fn take_command(tray: &StatusTray) -> ControlSurfaceCommand {
    let mut command = None;
    wait_until("tray command", || {
        command = tray.take_command();
        command.is_some()
    });
    command.expect("command should be present")
}

/// This test owns the well-known watcher name, so it must run in a private `dbus-run-session`.
#[test]
#[ignore = "requires a private D-Bus session; CI invokes it with dbus-run-session"]
fn hosted_protocol_actions_and_watcher_recovery() {
    let host = Watcher::start(true);
    let tray = StatusTray::new().expect("hosted tray should register");
    let service_name = host.wait_for_registration(1);
    let client = connection::Builder::session()
        .expect("client bus builder")
        .method_timeout(TIMEOUT)
        .build()
        .expect("client connection");

    let item = Proxy::new(&client, service_name.as_str(), SNI_PATH, SNI_INTERFACE)
        .expect("StatusNotifierItem proxy");
    assert_eq!(
        item.get_property::<String>("Title").expect("item title"),
        "Honk300 controls"
    );
    assert!(item
        .get_property::<String>("IconName")
        .expect("icon name")
        .is_empty());
    let pixmaps: Vec<(i32, i32, Vec<u8>)> = item
        .get_property("IconPixmap")
        .expect("embedded icon pixmap");
    assert_eq!((pixmaps[0].0, pixmaps[0].1), (36, 36));
    assert_eq!(pixmaps[0].2.len(), 36 * 36 * 4);

    let menu = Proxy::new(&client, service_name.as_str(), MENU_PATH, MENU_INTERFACE)
        .expect("dbusmenu proxy");
    let (_, layout): (u32, LayoutTuple) = menu
        .call("GetLayout", &(0_i32, -1_i32, Vec::<String>::new()))
        .expect("menu layout");
    let configure = find_menu_item(layout, "Configure Honk300…").expect("Configure menu item");
    let (_, layout): (u32, LayoutTuple) = menu
        .call("GetLayout", &(0_i32, -1_i32, Vec::<String>::new()))
        .expect("menu layout");
    let quit = find_menu_item(layout, "Quit Honk300").expect("Quit menu item");

    for (id, expected) in [
        (configure, ControlSurfaceCommand::Configure),
        (quit, ControlSurfaceCommand::Quit),
    ] {
        menu.call::<_, _, ()>(
            "Event",
            &(id, "clicked".to_owned(), OwnedValue::from(0_u8), 0_u32),
        )
        .expect("menu click");
        assert_eq!(take_command(&tray), expected);
    }

    host.close();
    std::thread::sleep(Duration::from_millis(100));
    assert!(!tray.is_closed());
    let recovered_host = Watcher::start(true);
    recovered_host.wait_for_registration(1);
    assert!(!tray.is_closed());
    drop(tray);
    recovered_host.close();
}

#[test]
#[ignore = "requires a private D-Bus session; CI invokes it with dbus-run-session"]
fn watcher_without_host_and_missing_watcher_fail_explicitly() {
    let watcher_without_host = Watcher::start(false);
    let error = StatusTray::new()
        .err()
        .expect("watcher without a host must not claim visible controls");
    assert!(error.to_string().contains("StatusNotifierHost"));
    watcher_without_host.close();

    let error = StatusTray::new()
        .err()
        .expect("missing watcher must remain an explicit unavailable result");
    assert!(error.to_string().contains("StatusNotifierWatcher"));
}
