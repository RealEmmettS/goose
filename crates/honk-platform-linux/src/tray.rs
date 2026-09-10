use honk_control::ControlSurfaceCommand;
use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::{MenuItem, StandardItem};
use std::sync::mpsc::{self, Receiver, Sender};
use tiny_skia::Pixmap;

use honk_control::icon::tray_pixmap;

#[derive(Debug)]
pub enum StatusTrayError {
    Asset(String),
    Service(ksni::Error),
}

impl std::fmt::Display for StatusTrayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asset(error) => write!(formatter, "embedded tray icon is invalid: {error}"),
            Self::Service(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for StatusTrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Asset(_) => None,
            Self::Service(error) => Some(error),
        }
    }
}

struct LinuxTray {
    commands: Sender<ControlSurfaceCommand>,
    icon: ksni::Icon,
}

impl LinuxTray {
    fn emit(&self, command: ControlSurfaceCommand) {
        let _ = self.commands.send(command);
    }
}

impl ksni::Tray for LinuxTray {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "honk300".into()
    }

    fn title(&self) -> String {
        "Goose controls".into()
    }

    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![self.icon.clone()]
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_name: String::new(),
            icon_pixmap: vec![self.icon.clone()],
            title: "Goose controls".into(),
            description: "Configure or update Goose, or send the goose home".into(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Configure Goose…".into(),
                activate: Box::new(|tray: &mut LinuxTray| {
                    tray.emit(ControlSurfaceCommand::Configure)
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Update Goose…".into(),
                activate: Box::new(|tray: &mut LinuxTray| tray.emit(ControlSurfaceCommand::Update)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Goose".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|tray: &mut LinuxTray| tray.emit(ControlSurfaceCommand::Quit)),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn watcher_online(&self) {
        eprintln!("honk300: Linux StatusNotifier host is available; controls restored.");
    }

    fn watcher_offline(&self, reason: ksni::OfflineReason) -> bool {
        eprintln!(
            "honk300: Linux StatusNotifier host became unavailable; CLI controls remain active ({reason:?})"
        );
        true
    }
}

/// Runtime-owned Linux StatusNotifierItem service.
pub struct StatusTray {
    commands: Receiver<ControlSurfaceCommand>,
    handle: Handle<LinuxTray>,
}

impl StatusTray {
    pub fn new() -> Result<Self, StatusTrayError> {
        let icon = status_icon()?;
        let (sender, commands) = mpsc::channel();
        let handle = LinuxTray {
            commands: sender,
            icon,
        }
        .spawn()
        .map_err(StatusTrayError::Service)?;
        eprintln!("honk300: Linux StatusNotifier controls are available.");
        Ok(Self { commands, handle })
    }

    pub fn take_command(&self) -> Option<ControlSurfaceCommand> {
        self.commands.try_recv().ok()
    }

    pub fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }
}

impl Drop for StatusTray {
    fn drop(&mut self) {
        self.handle.shutdown().wait();
    }
}

fn status_icon() -> Result<ksni::Icon, StatusTrayError> {
    let source = tray_pixmap().map_err(StatusTrayError::Asset)?;
    Ok(ksni::Icon {
        width: source.width() as i32,
        height: source.height() as i32,
        data: compose_tray_argb(&source),
    })
}

fn compose_tray_argb(source: &Pixmap) -> Vec<u8> {
    // StatusNotifier IconPixmap uses straight-alpha ARGB in network byte order.
    source
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let p = pixel.demultiply();
            [p.alpha(), p.red(), p.green(), p.blue()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{compose_tray_argb, status_icon, tray_pixmap, LinuxTray};
    use honk_control::ControlSurfaceCommand;
    use ksni::Tray;
    use std::sync::mpsc;

    #[test]
    fn status_icon_is_valid_contrasting_argb() {
        let source = tray_pixmap().expect("valid application tray artwork");
        let icon = status_icon().expect("valid embedded status icon");
        assert_eq!((icon.width, icon.height), (36, 36));
        assert_eq!(icon.data, compose_tray_argb(&source));
        assert!(icon.data.chunks_exact(4).any(|pixel| pixel[0] == 255));
        assert!(icon
            .data
            .chunks_exact(4)
            .any(|pixel| pixel[0] == 255 && pixel[1] > 220 && pixel[3] < 80));
    }

    #[test]
    fn status_notifier_pixels_keep_straight_alpha_and_network_channel_order() {
        let mut source = tiny_skia::Pixmap::new(2, 1).unwrap();
        source
            .data_mut()
            .copy_from_slice(&[100, 50, 25, 128, 0, 0, 0, 0]);
        let output = compose_tray_argb(&source);
        assert_eq!(output[0], 128);
        for (actual, expected) in output[1..4].iter().zip([200u8, 100, 50]) {
            assert!(actual.abs_diff(expected) <= 1);
        }
        assert_eq!(&output[4..], &[0, 0, 0, 0]);
    }

    #[test]
    fn menu_callbacks_emit_only_shared_commands() {
        let (sender, receiver) = mpsc::channel();
        let mut tray = LinuxTray {
            commands: sender,
            icon: status_icon().unwrap(),
        };
        let mut menu = tray.menu();
        let configure = match &mut menu[0] {
            ksni::MenuItem::Standard(item) => {
                std::mem::replace(&mut item.activate, Box::new(|_: &mut LinuxTray| {}))
            }
            _ => panic!("Configure should be a standard menu item"),
        };
        configure(&mut tray);
        let update = match &mut menu[1] {
            ksni::MenuItem::Standard(item) => {
                std::mem::replace(&mut item.activate, Box::new(|_: &mut LinuxTray| {}))
            }
            _ => panic!("Update should be a standard menu item"),
        };
        update(&mut tray);
        assert!(matches!(menu[2], ksni::MenuItem::Separator));
        let quit = match &mut menu[3] {
            ksni::MenuItem::Standard(item) => {
                std::mem::replace(&mut item.activate, Box::new(|_: &mut LinuxTray| {}))
            }
            _ => panic!("Quit should be a standard menu item"),
        };
        quit(&mut tray);
        assert_eq!(receiver.recv().unwrap(), ControlSurfaceCommand::Configure);
        assert_eq!(receiver.recv().unwrap(), ControlSurfaceCommand::Update);
        assert_eq!(receiver.recv().unwrap(), ControlSurfaceCommand::Quit);
        assert!(receiver.try_recv().is_err());
    }
}
