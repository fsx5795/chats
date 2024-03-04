#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use uuid::Uuid;
use sqlite;
use tauri::Manager;
use std::{collections::HashMap, env, fs, io::{Read, Write}, net::UdpSocket, path::PathBuf, thread};
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
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct UserInfo {
    ip: String,
    name: String,
}
static mut USERS: once_cell::sync::Lazy<HashMap<String, UserInfo>> = once_cell::sync::Lazy::new(|| {
    HashMap::new()
});
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ChatUser {
    id: String,
    name: String,
}
//UDP头像数据
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct HeadImg {
    name: String,
    contents: Vec<u8>,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum Values {
    Value(String),
    HeadImg(HeadImg),
}
//UDP数据
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct JsonData {
    id: String,
    types: String,
    values: Values,
}
//发送给界面的用户头像更改文件
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ModifyHead {
    id: String,
    path: String,
}
//发送给界面的用户聊天消息
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SendMsg {
    id: String,
    ip: String,
    name: String,
    msg: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Chatstory {
    iself: bool,
    name: String,
    msg: String,
}
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    //std::thread::sleep(std::time::Duration::from_micros(500_000));
    std::thread::sleep(std::time::Duration::from_secs(1));
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
        window.get_window("main").expect("no window labeled 'main' found").show().unwrap();
    }
    window.get_window("main").unwrap().set_always_on_top(false).unwrap();
    if cfg!(debug_assertions) {
        window.get_window("main").unwrap().open_devtools();
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
        let section = i.section(Some("User").to_owned()).unwrap();
        if let Some(value) = section.get("name") {
            return value.to_string();
        }
        /*
        for (_, prop) in i.iter() {
            for (k, v) in prop.iter() {
                if k == "name" {
                    return v.to_string();
                }
            }
        }
        */
    }
    String::new()
}
#[tauri::command]
fn set_user_info(name: String, img: String) {
    let mut curpath = env::current_exe().unwrap();
    curpath.pop();
    let mut inifile = curpath.clone();
    inifile.push("conf.ini");
    let mut conf = Ini::load_from_file(&inifile).unwrap();
    let section = conf.section_mut(Some("User").to_owned()).unwrap();
    section.insert("name".to_owned(), name.to_owned());
    conf.write_to_file(inifile).unwrap();
    let sendmsg = JsonData {
        id: UUID.to_string(),
        types: "name".to_string(),
        values: Values::Value(name),
    };
    let data = serde_json::to_string(&sendmsg).unwrap();
    if cfg!(debug_assertions) {
        SOCKET.send_to(&data.into_bytes(), "255.255.255.255:8080").unwrap();
    } else {
        SOCKET.send_to(&data.into_bytes(), "255.255.255.255:9527").unwrap();
    }
    if !img.is_empty() {
        let mut imgfile = curpath;
        let imgsour = PathBuf::from(img);
        imgfile.push(imgsour.file_name().unwrap());
        fs::copy(imgsour.clone(), imgfile.clone()).unwrap();
        let mut file = fs::File::open(imgfile).unwrap();
        let mut filedata = Vec::new();
        file.read_to_end(&mut filedata).unwrap();
        let headimg = HeadImg {
            name: imgsour.file_name().unwrap().to_string_lossy().to_string(),
            //contents: String::from_utf8(filedata).unwrap(),
            contents: filedata,
        };
        let sendmsg = JsonData {
            id: UUID.to_string(),
            types: "headimg".to_string(),
            values: Values::HeadImg(headimg),
        };
        let data = serde_json::to_string(&sendmsg).unwrap();
        if cfg!(debug_assertions) {
            SOCKET.send_to(&data.into_bytes(), "255.255.255.255:8080").unwrap();
        } else {
            SOCKET.send_to(&data.into_bytes(), "255.255.255.255:9527").unwrap();
        }
    }
}
#[tauri::command]
fn get_chats_history(id: String, handle: tauri::AppHandle) {
    unsafe {
        for (k, v) in USERS.iter() {
            if k.to_string() == id {
                let curname = v.name.clone();
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
                    }
                    handle.emit_to("main", "chatstory", Chatstory{ iself: iself, name: curname.clone(), msg: msg.to_string() }).unwrap();
                    true
                }).unwrap();
                break;
            }
        }
    }
}
#[tauri::command]
fn send_message(id: String, datetime: String, message: String) {
    let send_data = JsonData {
        id: UUID.to_string(),
        types: "chat".to_string(),
        values: Values::Value(message.clone()),
    };
    let data = serde_json::to_string(&send_data).unwrap();
    unsafe {
        for (k, v) in USERS.iter() {
            if k.to_string() == id {
                SOCKET.send_to(&data.into_bytes(), format!("{}", v.ip)).unwrap();
                let connection = get_db_connection();
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
                .theme(Some(tauri::Theme::Dark))
                .visible(false)
                .build()?;
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| menu_handle(app, event))
        .invoke_handler(tauri::generate_handler![close_splashscreen, get_user_name, set_user_info, get_chats_history, send_message])
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
        values: Values::Value(get_user_name()),
    };
    let data = serde_json::to_string(&name).unwrap();
    if cfg!(debug_assertions) {
        SOCKET.send_to(&data.into_bytes(), "255.255.255.255:8080")?;
    } else {
        SOCKET.send_to(&data.into_bytes(), "255.255.255.255:9527")?;
    }
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
        let ipstr = addr.to_string();
        let pos = ipstr.find(':').unwrap();
        let ipstr = &ipstr[..pos];
        //联系人上线或修改用户名
        if jsonvalue.types == "name" {
            if let Values::Value(strval) = jsonvalue.values {
                handle.emit_to("main", "ipname", ChatUser{ id: jsonvalue.id.clone(), name: strval.clone(), }).unwrap();
                let us = UserInfo {
                    ip: addr.to_string(),
                    name: strval
                };
                unsafe {
                    USERS.insert(jsonvalue.id, us);
                }
            };
        } else if jsonvalue.types == "headimg" {
            unsafe {
                for (k, v) in USERS.iter() {
                    if k.to_string() == jsonvalue.id {
                        if v.ip != ipstr.to_string() {
                            let us = UserInfo {
                                ip: ipstr.to_string(),
                                name: v.name.clone(),
                            };
                            USERS.insert(jsonvalue.id.clone(), us);
                        }
                    }
                }
            }
            if let Values::HeadImg(headimg) = jsonvalue.values {
                let mut curpath = env::current_exe().unwrap();
                curpath.pop();
                curpath.push(headimg.name);
                let mut file = fs::File::open(curpath.clone()).unwrap();
                file.write_all(&headimg.contents).unwrap();
                handle.emit_to("main", "userhead", ModifyHead{ id: jsonvalue.id, path: curpath.to_string_lossy().to_string() }).unwrap();
            }
        } else if jsonvalue.types == "chat" {
            let mut name = String::new();
            unsafe {
                for (k, v) in USERS.iter() {
                    if *k == jsonvalue.id {
                        if v.ip != ipstr.to_string() {
                            let us = UserInfo {
                                ip: ipstr.to_string(),
                                name: v.name.clone(),
                            };
                            USERS.insert(jsonvalue.id.clone(), us);
                        }
                        name = v.name.clone();
                    }
                }
            }
            if let Values::Value(strval) = jsonvalue.values {
                handle.emit_to("main", "chats", SendMsg{ id: jsonvalue.id.clone(), ip: ipstr.to_string(), name, msg: strval.clone() }).unwrap();
                let connection = get_db_connection();
                let now: DateTime<Local> = chrono::Local::now();
                let datetime = now.format("%Y-%m-%d %H:%M:%S");
                let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, chatmsg) VALUES ('{}', '{}', '{}', '{}');", jsonvalue.id, UUID.to_string(), datetime, strval);
                connection.execute(query).unwrap();
            };
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