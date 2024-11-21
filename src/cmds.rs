use std::io::ErrorKind;
use std::io::Read;
pub use tauri::Manager;
pub use tauri::Emitter;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Chatstory {
    iself: bool,
    name: String,
    types: String,
    msg: String,
}
#[tauri::command]
pub async fn close_splashscreen(window: tauri::Window) -> () {
    std::thread::sleep(std::time::Duration::from_secs(1));
    if let Some(splashscreen) = window.get_webview_window("splashscreen") {
        splashscreen.close().unwrap();
        window
            .get_webview_window("main")
            .expect("no window labeled 'main' found")
            .show()
            .unwrap();
    }
    window
        .get_webview_window("main")
        .unwrap()
        .set_always_on_top(false)
        .unwrap();
    #[cfg(debug_assertions)]
    window.get_webview_window("main").unwrap().open_devtools();
}
#[tauri::command]
pub fn get_user_name(id: String, connection: tauri::State<crate::SqlConArc>) -> String {
    crate::update_ipaddr(&id, "", &connection)
}
#[tauri::command]
pub fn set_admin_info(
    name: String,
    img: String,
    handle: tauri::AppHandle,
    uid: tauri::State<std::sync::Arc<uuid::Uuid>>,
    socket: tauri::State<crate::UdpArc>,
) -> () {
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
    let sendmsg = crate::JsonData::new(&uid.to_string(), "name", crate::Values::Value(name));
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
            let sendmsg = crate::JsonData::new(
                &uid.to_string(),
                "headimg",
                crate::Values::HeadImg {
                    status: String::from("start"),
                    contents: vec![],
                },
            );
            let data = serde_json::to_string(&sendmsg).unwrap();
            socket
                .send_to(&data.into_bytes(), "234.0.0.0:9527")
                .unwrap();
            for chunk in filedata.chunks(512) {
                std::thread::sleep(std::time::Duration::from_micros(10_000));
                let sendmsg = crate::JsonData::new(
                    &uid.to_string(),
                    "headimg",
                    crate::Values::HeadImg {
                        status: String::from("data"),
                        contents: chunk.to_vec(),
                    },
                );
                let data = serde_json::to_string(&sendmsg).unwrap();
                socket
                    .send_to(&data.into_bytes(), "234.0.0.0:9527")
                    .unwrap();
            }
            std::thread::sleep(std::time::Duration::from_micros(10_000));
            let sendmsg = crate::JsonData::new(
                &uid.to_string(),
                "headimg",
                crate::Values::HeadImg {
                    status: String::from("end"),
                    contents: vec![],
                },
            );
            let data = serde_json::to_string(&sendmsg).unwrap();
            socket
                .send_to(&data.into_bytes(), "234.0.0.0:9527")
                .unwrap();
        });
        section.insert("image".to_owned(), imgfile.to_string_lossy().to_string());
    }
    conf.write_to_file(inifile).unwrap();
}
#[tauri::command]
pub fn load_finish(
    handle: tauri::AppHandle,
    uid: tauri::State<crate::UidArc>,
    socket: tauri::State<crate::UdpArc>,
) -> () {
    let data = crate::get_admin_info_json(handle, &uid);
    if let Some(data) = data {
        socket
            .send_to(&data.into_bytes(), "234.0.0.0:9527")
            .unwrap();
    };
}
#[tauri::command]
pub fn get_admin_info(handle: tauri::AppHandle) -> String {
    let mut inifile = std::env::current_exe().unwrap();
    inifile.pop();
    inifile.push("conf.ini");
    if inifile.exists() {
        let i = ini::Ini::load_from_file(inifile).unwrap();
        let section = i.section(Some("Admin").to_owned()).unwrap();
        let mut name = String::new();
        let mut image = String::new();
        if let Some(value) = section.get("name") {
            name = value.to_owned();
        }
        if let Some(value) = section.get("image") {
            image = value.to_owned();
        }
        let json = crate::AdminInfo { name, image };
        return serde_json::to_string(&json).unwrap();
    } else {
        if let Err(_) = std::fs::File::create(&inifile) {
            handle.emit_to("main", "error", "用户信息保存失败").unwrap();
        }
        let id = uuid::Uuid::new_v4();
        let mut conf = ini::Ini::new();
        conf.with_section(Some("Admin")).set("id", id);
        conf.write_to_file(inifile).unwrap();
    }
    String::new()
}
#[tauri::command]
pub fn get_chats_history(
    id: String,
    handle: tauri::AppHandle,
    connection: tauri::State<crate::SqlConArc>,
    uid: tauri::State<crate::UidArc>,
) -> () {
    let query = format!("SELECT name FROM userinfo WHERE userid = '{}';", id);
    let connect = connection.lock().unwrap();
    connect
        .iterate(query, |pairs| {
            for &(_, value) in pairs.iter() {
                value.unwrap();
                let query = format!(
                    "SELECT * FROM chatshistory WHERE uuid = '{}' OR targetId = '{}';",
                    id, id
                );
                connect
                    .iterate(query, |pairs| {
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
                        handle
                            .emit_to(
                                "main",
                                "chatstory",
                                self::Chatstory {
                                    iself,
                                    name: value.unwrap().to_string(),
                                    types,
                                    msg: msg.to_owned(),
                                },
                            )
                            .unwrap();
                        true
                    })
                    .unwrap();
            }
            true
        })
        .unwrap();
}
#[tauri::command]
pub fn send_message(
    id: String,
    datetime: String,
    message: String,
    connection: tauri::State<crate::SqlConArc>,
    uid: tauri::State<crate::UidArc>,
    socket: tauri::State<crate::UdpArc>,
) -> () {
    let send_data = crate::JsonData::new(
        &uid.to_string(),
        "chat",
        crate::Values::Value(message.clone()),
    );
    let data = serde_json::to_string(&send_data).unwrap();
    let connect = connection.lock().unwrap();
    let query = format!("SELECT ip FROM userinfo WHERE userid = '{}';", id);
    connect
        .iterate(query, |pairs| {
            for &(_, value) in pairs.iter() {
                socket
                    .send_to(&data.clone().into_bytes(), format!("{}", value.unwrap()))
                    .unwrap();
            }
            true
        })
        .unwrap();
    let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", uid.to_string(), id, datetime, "text", message);
    connect.execute(query).unwrap();
}
#[tauri::command]
pub fn send_file(
    id: String,
    datetime: String,
    types: String,
    path: String,
    connection: tauri::State<crate::SqlConArc>,
    uid: tauri::State<crate::UidArc>,
    socket: tauri::State<crate::UdpArc>,
) -> () {
    let mut path = path;
    if types == "image" {
        let imgpath = crate::PathBuf::from(&path);
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
                        let sendmsg = crate::JsonData::new(&uid.to_string(), "chat", crate::Values::FileData { filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("start"), contents: vec![] });
                        let data = serde_json::to_string(&sendmsg).unwrap();
                        socket.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                        for chunk in buffer.chunks(512) {
                            std::thread::sleep(std::time::Duration::from_micros(10_000));
                            let sendmsg = crate::JsonData::new(&uid.to_string(), "chat", crate::Values::FileData {filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("data"), contents: chunk.to_vec()});
                            let data = serde_json::to_string(&sendmsg).unwrap();
                            socket.send_to(&data.into_bytes(), format!("{}", value.unwrap())).unwrap();
                        }
                        std::thread::sleep(std::time::Duration::from_micros(10_000));
                        let sendmsg = crate::JsonData::new(&uid.to_string(), "chat", crate::Values::FileData {filename: filesour.file_name().unwrap().to_string_lossy().to_string(), types: types.clone(), status: String::from("end"), contents: vec![]});
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
pub fn show_file(path: String) -> () {
    let mut path = crate::PathBuf::from(path);
    path.pop();
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .unwrap();
}
#[tauri::command]
pub fn close_window(uid: tauri::State<crate::UidArc>, socket: tauri::State<crate::UdpArc>) -> () {
    let send_data = crate::JsonData::new(
        &uid.to_string(),
        "events",
        crate::Values::Value("closed".to_owned()),
    );
    let data = serde_json::to_string(&send_data).unwrap();
    socket
        .send_to(&data.into_bytes(), "234.0.0.0:9527")
        .unwrap();
}
