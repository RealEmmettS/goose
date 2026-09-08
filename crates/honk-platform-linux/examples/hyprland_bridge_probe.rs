#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
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
