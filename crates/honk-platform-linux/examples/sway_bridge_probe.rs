#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("GITHUB_ACTIONS").as_deref() != Ok("true") {
        return Err("Use only the disposable native Sway fixture".into());
    }
    let mut connection = honk_platform_linux::sway::Connection::connect()?;
    let frame = connection.snapshot()?;
    let windows: Vec<_> = frame.windows.iter().map(|window| serde_json::json!({
        "id":window.id,"pid":window.pid,"title":window.title,"app":window.app,
        "geometry":[window.geometry.x,window.geometry.y,window.geometry.width as i32,window.geometry.height as i32],
        "visible":window.visible,"fullscreen":window.fullscreen
    })).collect();
    println!(
        "{}",
        serde_json::json!({"ok":true,"peer_pid":connection.peer().pid,
        "peer_uid":connection.peer().uid,"windows":windows,"fullscreen":frame.fullscreen(),
        "movement":false,"pointer_observation":false,"pointer_control":false})
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("The Sway native probe requires Linux");
}
