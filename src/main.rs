#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{collections::HashMap, io::{ErrorKind, Read}, path::PathBuf};
mod sqlsocket;
use sqlsocket::Manager;
use log4rs;
type SqlConArc = std::sync::Arc<std::sync::Mutex<sqlite::Connection>>;
type UdpArc = std::sync::Arc<std::net::UdpSocket>;
type UidArc = std::sync::Arc<uuid::Uuid>;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Chatstory {
    iself: bool,
    name: String,
    types: String,
    msg: String,
}
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) -> () {
    std::thread::sleep(std::time::Duration::from_secs(1));
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
        window.get_window("main").expect("no window labeled 'main' found").show().unwrap();
    }
    window.get_window("main").unwrap().set_always_on_top(false).unwrap();
    #[cfg(debug_assertions)]
    window.get_window("main").unwrap().open_devtools();
}
#[tauri::command]
fn get_user_name(id: String, connection: tauri::State<SqlConArc>) -> String {
    crate::sqlsocket::update_ipaddr(&id, "", &connection)
}
#[tauri::command]
fn set_admin_info(name: String, img: String, handle: tauri::AppHandle, uid: tauri::State<std::sync::Arc<uuid::Uuid>>, socket: tauri::State<UdpArc>) -> () {
    let mut curpath = std::env::current_exe().unwrap();
    curpath.pop();
    let mut inifile = curpath.clone();
    inifile.push("conf.ini");
    if !inifile.exists() {
        if let Err(_) = std::fs::File::create(&inifile) {
            return handle.emit_to("main", "error", "用户信息保存失败").unwrap();
        }
    }
    let mut conf = ini::Ini::load_from_file(&inifile).unwrap();
    let section = conf.section_mut(Some("Admin").to_owned()).unwrap();
    section.insert("name".to_owned(), name.to_owned());
    let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "name", crate::sqlsocket::Values::Value(name));
    let data = std::sync::Arc::new(serde_json::to_string(&sendmsg).unwrap());
    let dt = (*std::sync::Arc::clone(&data)).clone();
    socket.send_to(&dt.into_bytes(), "234.0.0.0:9527").unwrap();
    if !img.is_empty() {
        let mut imgfile = curpath;
        let imgsour = std::path::PathBuf::from(img);
        imgfile.push(imgsour.file_name().unwrap());
        std::fs::copy(&imgsour, &imgfile).unwrap();
        let mut file = std::fs::File::open(&imgfile).unwrap();
        let mut filedata = Vec::new();
        file.read_to_end(&mut filedata).unwrap();
        let uid = std::sync::Arc::clone(&uid);
        let socket = std::sync::Arc::clone(&socket);
        std::thread::spawn(move || {
            let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "headimg", crate::sqlsocket::Values::HeadImg{ status: String::from("start"), contents: vec![] });
            let data = serde_json::to_string(&sendmsg).unwrap();
            socket.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
            for chunk in filedata.chunks(512) {
                std::thread::sleep(std::time::Duration::from_micros(10_000));
                let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "headimg", crate::sqlsocket::Values::HeadImg{ status: String::from("data"), contents: chunk.to_vec() });
                let data = serde_json::to_string(&sendmsg).unwrap();
                socket.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
            }
            std::thread::sleep(std::time::Duration::from_micros(10_000));
            let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "headimg", crate::sqlsocket::Values::HeadImg{ status: String::from("end"), contents: vec![] });
            let data = serde_json::to_string(&sendmsg).unwrap();
            socket.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
        });
        section.insert("image".to_owned(), imgfile.to_string_lossy().to_string());
    }
    conf.write_to_file(inifile).unwrap();
}
#[tauri::command]
fn get_chats_history(id: String, handle: tauri::AppHandle, connection: tauri::State<SqlConArc>, uid: tauri::State<UidArc>) -> () {
    let query = format!("SELECT name FROM userinfo WHERE userid = '{}';", id);
    let connect = connection.lock().unwrap();
    connect.iterate(query, |pairs| {
        for &(_, value) in pairs.iter() {
            value.unwrap();
            let query = format!("SELECT * FROM chatshistory WHERE uuid = '{}' OR targetId = '{}';", id, id);
            connect.iterate(query, |pairs| {
                let mut iself = false;
                let mut types = String::new();
                let mut msg = String::new();
                for &(name, value) in pairs {
                    match name {
                        "uuid" => iself = value.unwrap() == uid.to_string(),
                        "type" => types = value.unwrap().to_owned(),
                        "chatmsg" => msg = value.unwrap().to_owned(),
                        _ => {}
                    }
                }
                handle.emit_to("main", "chatstory", Chatstory{ iself, name: value.unwrap().to_string(), types, msg: msg.to_owned() }).unwrap();
                true
            }).unwrap();
        }
        true
    }).unwrap();
}
#[tauri::command]
fn send_message(id: String, datetime: String, message: String, connection: tauri::State<SqlConArc>, uid: tauri::State<UidArc>, socket: tauri::State<UdpArc>) -> () {
    let send_data = crate::sqlsocket::JsonData::new(&uid.to_string(), "chat", crate::sqlsocket::Values::Value(message.clone()));
    let data = serde_json::to_string(&send_data).unwrap();
    let connect = connection.lock().unwrap();
    let query = format!("SELECT ip FROM userinfo WHERE userid = '{}';", id);
    connect.iterate(query, |pairs| {
        for &(_, value) in pairs.iter() {
            socket.send_to(&data.clone().into_bytes(), format!("{}", value.unwrap())).unwrap();
        }
        true
    }).unwrap();
    let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", uid.to_string(), id, datetime, "text", message);
    connect.execute(query).unwrap();
}
#[tauri::command]
fn send_file(id: String, datetime: String, types: String, path: String, connection: tauri::State<SqlConArc>, uid: tauri::State<UidArc>, socket: tauri::State<UdpArc>) -> () {
    let mut path = path;
    if types == "image" {
        let imgpath = PathBuf::from(&path);
        let mut curpath = std::env::current_exe().unwrap();
        curpath.pop();
        curpath.push(imgpath.file_name().unwrap());
        std::fs::copy(imgpath, &curpath).unwrap();
        path = curpath.into_os_string().into_string().unwrap();
    }
    if !path.is_empty() {
        let state = std::sync::Arc::clone(&connection);
        std::thread::scope(|s| {
            s.spawn(move || {
                let mut file = std::fs::File::open(&path).unwrap_or_else(|error|{
                    match error.kind() {
                        ErrorKind::NotFound => std::fs::File::create(&path).unwrap(),
                        _ => panic!("image error")
                    }
                });
                let connect = state.lock().unwrap();
                let query = format!("SELECT ip FROM userinfo WHERE userid = '{}';", id);
                connect.iterate(query, |pairs| {
                    for &(_, value) in pairs.iter() {
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        let filesour = std::path::PathBuf::from(&path);
                        let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "chat", crate::sqlsocket::Values::FileData { filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("start"), contents: vec![] });
                        let data = serde_json::to_string(&sendmsg).unwrap();
                        socket.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                        for chunk in buffer.chunks(512) {
                            std::thread::sleep(std::time::Duration::from_micros(10_000));
                            let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "chat", crate::sqlsocket::Values::FileData {filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("data"), contents: chunk.to_vec()});
                            let data = serde_json::to_string(&sendmsg).unwrap();
                            socket.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                        }
                        std::thread::sleep(std::time::Duration::from_micros(10_000));
                        let sendmsg = crate::sqlsocket::JsonData::new(&uid.to_string(), "chat", crate::sqlsocket::Values::FileData {filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("end"), contents: vec![]});
                        let data = serde_json::to_string(&sendmsg).unwrap();
                        socket.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                    }
                    true
                }).unwrap();
                let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", uid.to_string(), id, datetime, types, path);
                connect.execute(query).unwrap();
            });
        });
    }
}
#[tauri::command]
fn show_file(path : String) -> () {
    let mut path = PathBuf::from(path);
    path.pop();
    std::process::Command::new("explorer.exe").arg(path).spawn().unwrap();
}
#[tauri::command]
fn close_window(uid: tauri::State<UidArc>, socket: tauri::State<UdpArc>) -> () {
    let send_data = crate::sqlsocket::JsonData::new(&uid.to_string(), "events", crate::sqlsocket::Values::Value("closed".to_owned()));
    let data = serde_json::to_string(&send_data).unwrap();
    socket.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
}
fn main() -> () {
    let mut curpath = std::env::current_exe().unwrap();
    curpath.pop();
    //log
    let mut logpath = curpath.clone();
    logpath.push("output.log");
    let logfile = log4rs::append::file::FileAppender::builder().build(logpath).unwrap();
    let config = log4rs::config::Config::builder().appender(log4rs::config::Appender::builder().build("logfile", Box::new(logfile))).build(log4rs::config::Root::builder().appender("logfile").build(log::LevelFilter::Info)).unwrap();
    log4rs::init_config(config).unwrap();
    //网络文件缓冲区
    unsafe {
        crate::sqlsocket::FILEDATAS.get_or_init(|| HashMap::new());
    }
    //database connection
    let mut dbfile = curpath.clone();
    dbfile.push("chats.db");
    let dbexists = dbfile.exists();
    if !dbexists {
        if let Err(err) = std::fs::File::create(&dbfile) {
            log::error!("{}", err);
            return;
        };
    }
    let connection = sqlite::open(dbfile.as_path()).unwrap_or_else(|err| {
        log::error!("{}", err);
        panic!("{:?}", err)
    });
    let connection = std::sync::Arc::new(std::sync::Mutex::new(connection));
    if !dbexists {
        let query = "CREATE TABLE IF NOT EXISTS userinfo (userid TEXT, name TEXT, ip TEXT, imgpath VARCHAR(200))";
        if let Err(err) = connection.lock().unwrap().execute(query) {
            log::error!("{}", err)
        };
        let query = "CREATE TABLE IF NOT EXISTS chatshistory (uuid TEXT, targetId TEXT, chattime DATETIME, type TEXT, chatmsg VARCHAR(200))";
        if let Err(err) = connection.lock().unwrap().execute(query) {
            log::error!("{}", err)
        };
    }
    //uuid
    let mut inifile = curpath;
    inifile.push("conf.ini");
    let mut uid = std::sync::Arc::new(uuid::Uuid::new_v4());
    if inifile.exists() {
        match ini::Ini::load_from_file(&inifile) {
            Ok(i) => {
                for (_, prop) in &i {
                    prop.iter().find(|&(k, _)| k == "id").map(| (_, v) | uid = std::sync::Arc::new(uuid::Uuid::parse_str(v).unwrap()));
                }
            }
            Err(err) => log::error!("{}", err)
        }
    }
    let mut conf = ini::Ini::new();
    conf.with_section(Some("Admin")).set("id", *uid);
    if let Err(err) = conf.write_to_file(inifile) {
        log::error!("{}", err)
    };
    //udp socket
    let socket = std::net::UdpSocket::bind("0.0.0.0:9527").unwrap_or_else(|err| {
        log::error!("{}", err);
        panic!("{:?}", err)
    });
    let socket = std::sync::Arc::new(socket);
    let multiaddr = std::net::Ipv4Addr::new(234, 0, 0, 0);
    let interface = std::net::Ipv4Addr::new(0, 0, 0, 0);
    if let Err(err) = socket.join_multicast_v4(&multiaddr, &interface) {
        log::error!("{}", err);
        return
    };
    if let Err(err) = socket.set_multicast_loop_v4(false) {
        log::error!("{}", err);
        return
    };
    //系统托盘
    let quit = tauri::CustomMenuItem::new("quit".to_owned(), "关闭窗口");
    let hide = tauri::CustomMenuItem::new("hide".to_owned(), "隐藏窗口");
    let tray_menu = tauri::SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(hide);
    let system_tray = tauri::SystemTray::new().with_menu(tray_menu);
    let connect = std::sync::Arc::clone(&connection);
    let id = std::sync::Arc::clone(&uid);
    let udp = std::sync::Arc::clone(&socket);
    tauri::Builder::default()
        .setup(|app| {
            let apphandle = app.app_handle();
            std::thread::spawn(move || {
                if let Err(err) = crate::sqlsocket::init_socket(apphandle, connect, id, udp) {
                    log::error!("{}", err)
                };
            });
            tauri::WindowBuilder::new(app, "splashscreen", tauri::WindowUrl::App("splashscreen.html".parse().unwrap()))
                .decorations(false)
                .always_on_top(true)
                .theme(Some(tauri::Theme::Dark))
                .visible(false)
                .build()?;
            Ok(())
        })
        .manage(connection)
        .manage(uid)
        .manage(socket)
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| menu_handle(app, event))
        .invoke_handler(tauri::generate_handler![close_splashscreen, crate::sqlsocket::load_finish, crate::sqlsocket::get_admin_info, get_user_name, set_admin_info, get_chats_history, send_message, send_file, show_file, close_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
fn menu_handle(app_handle: &tauri::AppHandle, event: tauri::SystemTrayEvent) -> () {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => std::process::exit(0),
            "hide" => {
                let item_handle = app_handle.tray_handle().get_item(&id);
                let window = app_handle.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                    item_handle.set_title("显示窗口").unwrap()
                } else {
                    window.show().unwrap();
                    item_handle.set_title("隐藏窗口").unwrap()
                }
            },
            _ => {}
        },
        _ => {}
    }
}