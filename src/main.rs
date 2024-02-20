#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{Manager, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use std::net::UdpSocket;
#[derive(Clone, serde::Serialize)]
struct Payload {}
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    std::thread::sleep(std::time::Duration::from_micros(500_000));
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    /*
    std::thread::sleep(std::time::Duration::from_micros(500_000));
    let handle = window.app_handle();
    handle.emit_to("splashscreen", "msg", Payload {}).unwrap();
    */
    window.get_window("main").expect("no window labeled 'main' found").show().unwrap();
}
fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "关闭窗口");
    let hide = CustomMenuItem::new("hide".to_string(), "隐藏窗口");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        /*
        .setup(|app| {
            tauri::WindowBuilder::new(app, "splashscreen", tauri::WindowUrl::App("splashscreen.html".into()))
                .decorations(false)
                .always_on_top(true)
                .build()?;
            Ok(())
        })
*/
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| menu_handle(app, event))
        .invoke_handler(tauri::generate_handler![close_splashscreen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    init_socket().unwrap();
}
fn init_socket() -> std::io::Result<()> {
    let socket = UdpSocket::bind("192.168.0.204:9527")?;
    socket.set_broadcast(true)?;
    socket.set_multicast_loop_v4(true)?;
    //&str转Vec<u8>
    let data = "abc".as_bytes().to_vec();
    socket.send_to(&data, "255.255.255.255:9527")?;
    let mut buf = [0; 10];
    let (amt, _) = socket.recv_from(&mut buf)?;
    //Vec<u8>转&str
    println!("{}", std::str::from_utf8(&buf[..amt]).unwrap());
    Ok(())
}
fn menu_handle(app_handle: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => std::process::exit(0),
            "hide" => {
                let item_handle = app_handle.tray_handle().get_item(&id);
                let window = app_handle.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                    item_handle.set_title("显示窗口").unwrap();
                } else {
                    window.show().unwrap();
                    item_handle.set_title("隐藏窗口").unwrap();
                }
            }
            _ => {}
        },
        _ => {}
    }
}