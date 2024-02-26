#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{Manager, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use std::fmt::Formatter;
use std::net::UdpSocket;
use std::thread;
use std::env;
use ini::Ini;
use once_cell::sync::Lazy;
static SOCKET: Lazy<UdpSocket> = Lazy::new(|| {
    UdpSocket::bind("0.0.0.0:9527").unwrap()
});
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct IpName {
    ip: String,
    name: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum Values {
    Msg(String),
    ChatMsg(SendMsg),
}
impl Into<String> for Values {
    fn into(self) -> String {
        match self {
            Values::Msg(msg) => msg,
            _ => String::from("")
        }
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct JsonData {
    types: String,
    values: Values,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SendMsg {
    ip: String,
    msg: String,
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
#[tauri::command]
fn get_chats_history(ip: String) {
    let mut dbfile = env::current_exe().unwrap();
    dbfile.pop();
    dbfile.push("chats.db");
    let dbexists = dbfile.exists();
    if dbexists {
        std::fs::File::create(&dbfile).unwrap();
    }
    let connection = sqlite::open(dbfile.as_path()).unwrap();
    let query = "CREATE TABLE IF NOT EXISTS chatshistory (name TEXT, chat VARCHAR(200))";
    connection.execute(query).unwrap();
    let query = format!("SELECT * FROM chatshistory WHERE name = '{}';", ip);
    let query = query.as_str();
    connection.iterate(query, |pairs| {
        for &(name, value) in pairs.iter() {
            println!("{}:{}", name, value.unwrap());
        }
        true
    }).unwrap();
}
#[tauri::command]
fn send_message(ip: String, message: String) {
    let sendmsg = SendMsg {
        ip: ip.clone(),
        msg: message,
    };
    let send_data = JsonData {
        types: "send".to_string(),
        values: Values::ChatMsg(sendmsg),
    };
    let data = serde_json::to_string(&send_data).unwrap();
    println!("{}", data);
    SOCKET.send_to(&data.into_bytes(), format!("{}", ip)).unwrap();
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
            let apphanle = app.app_handle().clone();
            thread::spawn(move || { init_socket(apphanle).unwrap() });
            tauri::WindowBuilder::new(app, "splashscreen", tauri::WindowUrl::App("splashscreen.html".into()))
                .decorations(false)
                .always_on_top(true)
                .build()?;
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| menu_handle(app, event))
        .invoke_handler(tauri::generate_handler![close_splashscreen, get_user_name, get_chats_history, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
fn init_socket(handle: tauri::AppHandle) -> std::io::Result<()> {
    SOCKET.set_broadcast(true)?;
    SOCKET.set_multicast_loop_v4(true)?;
    let name = JsonData {
        types: "name".to_string(),
        values: Values::Msg(get_user_name()),
    };
    let data = serde_json::to_string(&name).unwrap();
    SOCKET.send_to(&data.into_bytes(), "255.255.255.255:8080")?;
    loop {
        let mut buf = [0; 1000];
        let (amt, addr) = SOCKET.recv_from(&mut buf)?;
        let jsonvalue = serde_json::from_str(&String::from_utf8_lossy(&buf[..amt]).to_string());
        if jsonvalue.is_err() {
            println!("json err:{}", jsonvalue.unwrap());
            continue;
        }
        println!("{}", &String::from_utf8_lossy(&buf[..amt]).to_string());
        let jsonvalue = serde_json::from_value::<JsonData>(jsonvalue.unwrap());
        if jsonvalue.is_err() {
            println!("JsonData err");
            continue;
        }
        let jsonvalue = jsonvalue.unwrap();
        println!("{}", jsonvalue.types);
        if jsonvalue.types == "name" {
            handle.emit_to("main", "ipname", IpName{ ip: addr.to_string(), name: jsonvalue.values.into(), }).unwrap();
        } else if jsonvalue.types == "send" {
            match jsonvalue.values {
                Values::ChatMsg(chatmsg) => {
                    chatmsg.ip.split(':');
                    let _ = handle.emit_to("main", "chats", SendMsg{ ip: chatmsg.ip, msg: chatmsg.msg });
                },
                _ => ()
            };
        }
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