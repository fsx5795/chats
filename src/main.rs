#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{io::{ErrorKind, Read}, path::PathBuf};
mod sqlsocket;
use sqlsocket::Manager;
struct CusState(std::sync::Arc<std::sync::Mutex<sqlite::Connection>>);
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
    /*
    std::thread::sleep(std::time::Duration::from_micros(500_000));
    let handle = window.app_handle();
    handle.emit_to("splashscreen", "msg", Payload {}).unwrap();
    */
}
#[tauri::command]
fn get_user_name(id: String, state: tauri::State<CusState>) -> String {
    sqlsocket::update_ipaddr(&id, "", &state.0)
}
#[tauri::command]
fn set_admin_info(name: String, img: String, handle: tauri::AppHandle) -> () {
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
    let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "name", sqlsocket::Values::Value(name));
    let data = serde_json::to_string(&sendmsg).unwrap();
    sqlsocket::SOCKET.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
    if !img.is_empty() {
        let mut imgfile = curpath;
        let imgsour = std::path::PathBuf::from(img);
        imgfile.push(imgsour.file_name().unwrap());
        std::fs::copy(&imgsour, &imgfile).unwrap();
        let mut file = std::fs::File::open(&imgfile).unwrap();
        let mut filedata = Vec::new();
        file.read_to_end(&mut filedata).unwrap();
        std::thread::spawn(move || {
            let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "headimg", sqlsocket::Values::HeadImg{ status: String::from("start"), contents: vec![] });
            let data = serde_json::to_string(&sendmsg).unwrap();
            sqlsocket::SOCKET.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
            for chunk in filedata.chunks(512) {
                std::thread::sleep(std::time::Duration::from_micros(500_000));
                let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "headimg", sqlsocket::Values::HeadImg{ status: String::from("data"), contents: chunk.to_vec() });
                let data = serde_json::to_string(&sendmsg).unwrap();
                sqlsocket::SOCKET.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
            }
            std::thread::sleep(std::time::Duration::from_micros(500_000));
            let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "headimg", sqlsocket::Values::HeadImg{ status: String::from("end"), contents: vec![] });
            let data = serde_json::to_string(&sendmsg).unwrap();
            sqlsocket::SOCKET.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
        });
        section.insert("image".to_owned(), imgfile.to_string_lossy().to_string());
    }
    conf.write_to_file(inifile).unwrap();
}
#[tauri::command]
fn get_chats_history(id: String, handle: tauri::AppHandle, state: tauri::State<CusState>) -> () {
    let query = format!("SELECT name FROM userinfo WHERE userid = '{}';", id);
    let connect = state.0.lock().unwrap();
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
                        "uuid" => iself = value.unwrap() == sqlsocket::UUID.to_string(),
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
fn send_message(id: String, datetime: String, message: String, state: tauri::State<CusState>) -> () {
    let send_data = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "chat", sqlsocket::Values::Value(message.clone()));
    let data = serde_json::to_string(&send_data).unwrap();
    let connect = state.0.lock().unwrap();
    let query = format!("SELECT ip FROM userinfo WHERE userid = '{}';", id);
    connect.iterate(query, |pairs| {
        for &(_, value) in pairs.iter() {
            sqlsocket::SOCKET.send_to(&data.clone().into_bytes(), format!("{}", value.unwrap())).unwrap();
        }
        true
    }).unwrap();
    let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", sqlsocket::UUID.to_owned(), id, datetime, "text", message);
    connect.execute(query).unwrap();
}
#[tauri::command]
fn send_file(id: String, datetime: String, types: String, path: String, state: tauri::State<CusState>) -> () {
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
        let state = std::sync::Arc::clone(&state.0);
        std::thread::spawn(move || {
            match std::fs::File::open(&path) {
                Ok(mut file) => {
                    let connect = state.lock().unwrap();
                    let query = format!("SELECT ip FROM userinfo WHERE userid = '{}';", id);
                    connect.iterate(query, |pairs| {
                        for &(_, value) in pairs.iter() {
                            let mut buffer = Vec::new();
                            file.read_to_end(&mut buffer).unwrap();
                            let filesour = std::path::PathBuf::from(&path);
                            let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "chat", sqlsocket::Values::FileData { filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("start"), contents: vec![] });
                            let data = serde_json::to_string(&sendmsg).unwrap();
                            sqlsocket::SOCKET.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                            for chunk in buffer.chunks(512) {
                                std::thread::sleep(std::time::Duration::from_micros(100_000));
                                let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "chat", sqlsocket::Values::FileData {filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("data"), contents: chunk.to_vec()});
                                let data = serde_json::to_string(&sendmsg).unwrap();
                                sqlsocket::SOCKET.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                            }
                            std::thread::sleep(std::time::Duration::from_micros(100_000));
                            let sendmsg = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "chat", sqlsocket::Values::FileData {filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("end"), contents: vec![]});
                            let data = serde_json::to_string(&sendmsg).unwrap();
                            sqlsocket::SOCKET.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                        }
                        true
                    }).unwrap();
                    let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", sqlsocket::UUID.to_owned(), id, datetime, types, path);
                    connect.execute(query).unwrap();
                }
                Err(error) => match error.kind() {
                    ErrorKind::NotFound => {}
                    _ => {}
                }
            }
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
fn close_window() -> () {
    let send_data = sqlsocket::JsonData::new(&sqlsocket::UUID.to_string(), "events", sqlsocket::Values::Value("closed".to_owned()));
    let data = serde_json::to_string(&send_data).unwrap();
    sqlsocket::SOCKET.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
}
fn main() -> () {
    let mut dbfile = std::env::current_exe().unwrap();
    dbfile.pop();
    dbfile.push("chats.db");
    let dbexists = dbfile.exists();
    if !dbexists {
        std::fs::File::create(&dbfile).unwrap();
    }
    let connection =  std::sync::Arc::new(std::sync::Mutex::new(sqlite::open(dbfile.as_path()).unwrap()));
    if !dbexists {
        let query = "CREATE TABLE IF NOT EXISTS userinfo (userid TEXT, name TEXT, ip TEXT, imgpath VARCHAR(200))";
        connection.lock().unwrap().execute(query).unwrap();
        let query = "CREATE TABLE IF NOT EXISTS chatshistory (uuid TEXT, targetId TEXT, chattime DATETIME, type TEXT, chatmsg VARCHAR(200))";
        connection.lock().unwrap().execute(query).unwrap();
    }
    let quit = tauri::CustomMenuItem::new("quit".to_owned(), "关闭窗口");
    let hide = tauri::CustomMenuItem::new("hide".to_owned(), "隐藏窗口");
    let tray_menu = tauri::SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(hide);
    let system_tray = tauri::SystemTray::new().with_menu(tray_menu);
    let connect = std::sync::Arc::clone(&connection);
    tauri::Builder::default()
        .setup(|app| {
            let apphanle = app.app_handle();
            std::thread::spawn(move || { sqlsocket::init_socket(apphanle, connect).unwrap() });
            tauri::WindowBuilder::new(app, "splashscreen", tauri::WindowUrl::App("splashscreen.html".parse().unwrap()))
                .decorations(false)
                .always_on_top(true)
                .theme(Some(tauri::Theme::Dark))
                .visible(false)
                .build()?;
            Ok(())
        })
        .manage(CusState(connection))
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| menu_handle(app, event))
        .invoke_handler(tauri::generate_handler![close_splashscreen, sqlsocket::load_finish, sqlsocket::get_admin_info, get_user_name, set_admin_info, get_chats_history, send_message, send_file, show_file, close_window])
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