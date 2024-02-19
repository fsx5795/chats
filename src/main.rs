#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::Manager;
use std::net::UdpSocket;
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    if let Some(splashscreen) = window.get_window("splashscreen") {
        std::thread::sleep(std::time::Duration::from_micros(500_000));
        splashscreen.close().unwrap();
    } else {
        println!("splash screen window not found");
    }
    std::thread::sleep(std::time::Duration::from_micros(500_000));
    window.get_window("main").expect("no window labeled 'main' found").show().unwrap();
}
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            tauri::WindowBuilder::new(app, "splashscreen", tauri::WindowUrl::App("abc.png".into()))
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![close_splashscreen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    init_socket().unwrap();
}
fn init_socket() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9527")?;
    socket.set_broadcast(true)?;
    //&str转Vec<u8>
    let data = "abc".as_bytes().to_vec();
    socket.send_to(&data, "255.255.255.255:9527")?;
    Ok(())
}