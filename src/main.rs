#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{Manager, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use std::net::IpAddr;
use std::net::UdpSocket;
use std::thread;
use std::env;
use ini::Ini;
#[derive(Clone, serde::Serialize)]
struct Ipname {
    ip: String,
    name: String,
}
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    std::thread::sleep(std::time::Duration::from_micros(500_000));
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
        window.get_window("main").expect("no window labeled 'main' found").show().unwrap();
    }
    /*
    std::thread::sleep(std::time::Duration::from_micros(500_000));
    let handle = window.app_handle();
    handle.emit_to("splashscreen", "msg", Payload {}).unwrap();
    */
}
#[tauri::command]
fn get_user_name() -> String {
    let mut cur_dir = env::current_exe().unwrap();
    cur_dir.pop();
    cur_dir.push("conf.ini");
    if cur_dir.exists() {
        let i = Ini::load_from_file(cur_dir).unwrap();
        for (_, prop) in i.iter() {
            for (k, v) in prop.iter() {
                if k == "name" {
                    return v.to_string();
                }
            }
        }
    }
    String::new()
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
        .setup(|app| {
            let mut apphanle = app.app_handle().clone();
            thread::spawn(move || { init_socket(apphanle).unwrap() });
            tauri::WindowBuilder::new(app, "splashscreen", tauri::WindowUrl::App("splashscreen.html".into()))
                .decorations(false)
                .always_on_top(true)
                .build()?;
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| menu_handle(app, event))
        .invoke_handler(tauri::generate_handler![close_splashscreen, get_user_name])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
fn init_socket(handle: tauri::AppHandle) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9527")?;
    socket.set_broadcast(true)?;
    socket.set_multicast_loop_v4(true)?;
    //&str转Vec<u8>
    let data = get_user_name().into_bytes();
    socket.send_to(&data, "255.255.255.255:8080")?;
    loop {
        let mut buf = [0; 10];
        let (amt, addr) = socket.recv_from(&mut buf)?;
        handle.emit_to("main", "ipname", Ipname { ip: addr.to_string(), name: String::from_utf8_lossy(&buf[..amt]).to_string() }).unwrap();
    }
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