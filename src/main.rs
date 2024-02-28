#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use uuid::Uuid;
use sqlite;
use tauri::Manager;
use std::{env, thread, collections::HashMap, net::UdpSocket};
use ini::Ini;
use once_cell;
use chrono::{self, DateTime, Local};
static SOCKET: once_cell::sync::Lazy<UdpSocket> = once_cell::sync::Lazy::new(|| {
    UdpSocket::bind("0.0.0.0:9527").unwrap()
});
static UUID: once_cell::sync::Lazy<Uuid> = once_cell::sync::Lazy::new(|| {
    let mut inifile = env::current_exe().unwrap();
    inifile.pop();
    inifile.push("conf.ini");
    if inifile.exists() {
        let i = Ini::load_from_file(&inifile).unwrap();
        for (_, prop) in i.iter() {
            for (k, v) in prop.iter() {
                if k == "id" {
                    return Uuid::parse_str(v).unwrap();
                }
            }
        }
    }
    let id = Uuid::new_v4();
    let mut conf = Ini::new();
    conf.with_section(Some("User")).set("id", id);
    conf.write_to_file(inifile).unwrap();
    id
});
static mut USERS: once_cell::sync::Lazy<HashMap<String, UserInfo>> = once_cell::sync::Lazy::new(|| {
    HashMap::new()
});
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct UserInfo {
    ip: String,
    name: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct JsonData {
    id: String,
    types: String,
    values: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SendMsg {
    ip: String,
    msg: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Chatstory {
    iself: bool,
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
    let mut inifile = env::current_exe().unwrap();
    inifile.pop();
    inifile.push("conf.ini");
    if inifile.exists() {
        let i = Ini::load_from_file(inifile).unwrap();
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
fn get_chats_history(ip: String, handle: tauri::AppHandle) {
    unsafe {
        for (k, v) in USERS.iter() {
            if v.ip == ip {
                let connection = get_db_connection();
                let query = format!("SELECT * FROM chatshistory WHERE uuid = '{}' OR targetId = '{}';", k, k);
                let query = query.as_str();
                connection.iterate(query, |pairs| {
                    let mut iself = false;
                    let mut msg = String::new();
                    for &(name, value) in pairs.iter() {
                        if name == "uuid" {
                            iself = value.unwrap() == UUID.to_string();
                        } else if name == "chatmsg" {
                            msg = value.unwrap().to_string();
                        }
                        println!("{}:{}", name, value.unwrap());
                    }
                    handle.emit_to("main", "chatstory", Chatstory{ iself: iself, msg: msg.to_string() }).unwrap();
                    true
                }).unwrap();
                break;
            }
        }
    }
}
#[tauri::command]
fn send_message(ip: String, datetime: String, message: String) {
    let send_data = JsonData {
        id: UUID.to_string(),
        types: "chat".to_string(),
        values: message.clone(),
    };
    let data = serde_json::to_string(&send_data).unwrap();
    SOCKET.send_to(&data.into_bytes(), format!("{}", ip)).unwrap();
    let connection = get_db_connection();
    unsafe {
        for (k, v) in USERS.iter() {
            if v.ip == ip {
                let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, chatmsg) VALUES ('{}', '{}', '{}', '{}');", UUID.to_string(), k, datetime, message);
                connection.execute(query).unwrap();
                break;
            }
        }
    }
}
fn main() {
    let quit = tauri::CustomMenuItem::new("quit".to_string(), "关闭窗口");
    let hide = tauri::CustomMenuItem::new("hide".to_string(), "隐藏窗口");
    let tray_menu = tauri::SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(hide);
    let system_tray = tauri::SystemTray::new().with_menu(tray_menu);
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
fn get_db_connection() -> sqlite::Connection {
    let mut dbfile = env::current_exe().unwrap();
    dbfile.pop();
    dbfile.push("chats.db");
    let dbexists = dbfile.exists();
    if !dbexists {
        std::fs::File::create(&dbfile).unwrap();
    }
    let connection = sqlite::open(dbfile.as_path()).unwrap();
    let query = "CREATE TABLE IF NOT EXISTS chatshistory (uuid TEXT, targetId TEXT, chattime DATETIME, chatmsg VARCHAR(200))";
    connection.execute(query).unwrap();
    connection
}
fn init_socket(handle: tauri::AppHandle) -> std::io::Result<()> {
    SOCKET.set_broadcast(true)?;
    SOCKET.set_multicast_loop_v4(true)?;
    let name = JsonData {
        id: UUID.to_string(),
        types: "name".to_string(),
        values: get_user_name(),
    };
    let data = serde_json::to_string(&name).unwrap();
    //SOCKET.send_to(&data.into_bytes(), "255.255.255.255:9527")?;
    SOCKET.send_to(&data.into_bytes(), "255.255.255.255:8080")?;
    loop {
        let mut buf = [0; 1000];
        let (amt, addr) = SOCKET.recv_from(&mut buf)?;
        let jsonvalue = serde_json::from_str(&String::from_utf8_lossy(&buf[..amt]).to_string());
        if jsonvalue.is_err() {
            println!("json err:{}", jsonvalue.unwrap());
            continue;
        }
        let jsonvalue = serde_json::from_value::<JsonData>(jsonvalue.unwrap());
        if jsonvalue.is_err() {
            println!("JsonData err");
            continue;
        }
        let jsonvalue = jsonvalue.unwrap();
        println!("{}", jsonvalue.types);
        if jsonvalue.types == "name" {
            handle.emit_to("main", "ipname", UserInfo{ ip: addr.to_string(), name: jsonvalue.values.clone(), }).unwrap();
            let us = UserInfo {
                ip: addr.to_string(),
                name: jsonvalue.values,
            };
            unsafe {
                USERS.insert(jsonvalue.id, us);
            }
        } else if jsonvalue.types == "chat" {
            let ipstr = addr.to_string();
            let pos = ipstr.find(':').unwrap();
            let ipstr = &ipstr[..pos];
            handle.emit_to("main", "chats", SendMsg{ ip: ipstr.to_string(), msg: jsonvalue.values.clone() }).unwrap();
            let connection = get_db_connection();
            let now: DateTime<Local> = chrono::Local::now();
            let datetime = now.format("%Y-%m-%d %H:%M:%S");
            let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, chatmsg) VALUES ('{}', '{}', '{}', '{}');", jsonvalue.id, UUID.to_string(), datetime, jsonvalue.values);
            connection.execute(query).unwrap();
        }
    }
}
fn menu_handle(app_handle: &tauri::AppHandle, event: tauri::SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
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