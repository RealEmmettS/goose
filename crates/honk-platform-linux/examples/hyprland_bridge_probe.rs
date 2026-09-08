#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|argument| argument == "--watch") {
        use std::io::Write;
        let observer = honk_platform_linux::hyprland::Observer::start()?;
        for _ in 0..400 {
            let frame = observer.snapshot();
            let fullscreen_window = frame.as_ref().and_then(|frame| frame.windows.iter()
                .find(|window| window.visible && window.fullscreen))
                .map(|window| serde_json::json!({"id":window.id,"pid":window.pid,"geometry":window.geometry}));
            println!(
                "{}",
                serde_json::json!({"observed":frame.is_some(),
                "failed":observer.failed(),"fullscreen":frame.as_ref().is_some_and(|frame|frame.fullscreen()),
                "fullscreen_window":fullscreen_window})
            );
            std::io::stdout().flush()?;
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        return Ok(());
    }
    let connection = honk_platform_linux::hyprland::Connection::connect()?;
    let frame = connection.snapshot()?;
    let windows: Vec<_> = frame
        .windows
        .iter()
        .map(|window| {
            serde_json::json!({
                "id":window.id,"pid":window.pid,"title":window.title,"app":window.app,
                "geometry":window.geometry,"visible":window.visible,"fullscreen":window.fullscreen,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({"ok":true,"peer_pid":connection.peer().pid,
        "peer_uid":connection.peer().uid,"version":connection.version().version,
        "fullscreen":frame.fullscreen(),"windows":windows,
        "movement":false,"pointer_observation":false,"pointer_control":false})
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("The native Hyprland probe requires Linux");
    std::process::exit(2);
}
