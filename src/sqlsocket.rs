use std::{collections::HashMap, io::Write, path::PathBuf};
pub use tauri::Manager;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AdminInfo {
    name: String,
    image: String
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ChatUser {
    id: String,
    name: String,
}
//发送给界面的用户头像更改文件
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ModifyHead {
    id: String,
    path: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SendFile {
    id: String,
    types: String,
    name: String,
    path: String,
}
//发送给界面的用户聊天消息
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SendMsg {
    id: String,
    name: String,
    msg: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Values {
    //消息
    Value(String),
    FileData{ filename: String, types: String, status: String, contents: Vec<u8> },
    HeadImg{ status: String, contents: Vec<u8> }
}
#[derive(Debug)]
struct CusErr {
    detail: String,
}
impl std::fmt::Display for CusErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.detail)
    }
}
impl std::error::Error for CusErr {
    /*
    fn description(&self) -> &str {
        &self.detail
    }
    */
}
//UDP数据
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonData {
    id: String,
    types: String,
    values: Values,
}
impl JsonData {
    pub fn new(id: &str, types: &str, values: Values) -> Self {
        JsonData {
            id: id.to_owned(),
            types: types.to_owned(),
            values
        }
    }
    fn anaslysis(&self, ipstr: &str, addr: &std::net::SocketAddr, handle: &tauri::AppHandle, connection: &std::sync::Arc<std::sync::Mutex<sqlite::Connection>>, uid: &std::sync::Arc<uuid::Uuid>, socket: &std::sync::Arc<std::net::UdpSocket>) -> Result<(), Box<dyn std::error::Error>> {
        static mut IDS: Vec<String> = Vec::new();
        match self.types.as_str() {
            //联系人上线或修改用户名
            "name" => {
                if let Values::Value(strval) = &self.values {
                    handle.emit_to("main", "ipname", ChatUser{ id: self.id.clone(), name: strval.clone(), })?;
                    let mut nothas = true;
                    let mut same = true;
                    let query = format!("SELECT * FROM userinfo WHERE userid = '{}';", self.id);
                    connection.lock().unwrap().iterate(query, |pairs| {
                        nothas = false;
                        for &(name, value) in pairs.iter() {
                            if name == "name" && value.unwrap() != strval {
                                same = false;
                                break;
                            }
                            if name == "ip" && value.unwrap() != addr.to_string() {
                                same = false;
                                break;
                            }
                        }
                        true
                    }).unwrap();
                    if !same {
                        let query = format!("UPDATE userinfo SET name = '{}', ip = '{}' WHERE userid = '{}';", strval, addr, self.id);
                        connection.lock().unwrap().execute(query).unwrap();
                    }
                    if nothas {
                        let query = format!("INSERT INTO userinfo (userid, name, ip) VALUES ('{}', '{}', '{}');", self.id, strval, addr);
                        connection.lock().unwrap().execute(query).unwrap();
                    }
                    unsafe {
                        if !IDS.contains(&self.id) {
                            IDS.push(self.id.clone());
                            let data = get_admin_info_json(handle.clone(), &uid);
                            socket.send_to(&data.into_bytes(), addr).unwrap();
                        }
                    }
                };
            }
            "headimg" => {
                if let Values::HeadImg{status, contents} = &self.values {
                    let mut curpath = std::env::current_exe()?;
                    curpath.pop();
                    curpath.push(&self.id);
                    match status.as_str() {
                        //"start" => { let _ = std::fs::File::create(&curpath)?; }
                        "start" => {
                            let _ = std::fs::File::create(&curpath)?;
                            unsafe {
                                FILEDATAS.get_mut().unwrap().insert(curpath, std::collections::VecDeque::new());
                            }
                        }
                        "data" => {
                            /*
                            let mut file = std::fs::OpenOptions::new().append(true).open(&curpath)?;
                            file.write_all(&contents)?;
                            */
                            unsafe {
                                FILEDATAS.get_mut().unwrap().get_mut(&curpath).unwrap().push_back(contents.to_vec());
                            }
                        }
                        "end" => {
                            let cp = curpath.clone();
                            std::thread::scope(|s| {
                                s.spawn(move || {
                                    let mut file = std::fs::OpenOptions::new().append(true).open(&cp).unwrap();
                                    unsafe {
                                        while FILEDATAS.get().unwrap().get(&cp).unwrap().len() > 0 {
                                            file.write_all(&FILEDATAS.get_mut().unwrap().get_mut(&cp).unwrap().pop_front().unwrap()).unwrap();
                                        }
                                        FILEDATAS.get_mut().unwrap().remove(&cp);
                                    }
                                    //handle.emit_to("main", "userfile", SendFile{ id: self.id.clone(), types: (*types).clone(), name, path: cp.to_string_lossy().to_string() }).unwrap();
                                    handle.emit_to("main", "userhead", ModifyHead{ id: self.id.clone(), path: cp.to_string_lossy().to_string() }).unwrap();
                                });
                            });
                            let query = format!("UPDATE userinfo SET ip = '{}', imgpath = '{}' WHERE userid = '{}';", ipstr, curpath.to_string_lossy().to_string(), self.id);
                            connection.lock().unwrap().execute(query)?;
                        }
                        _ => {}
                    }
                }
            }
            "chat" => {
                let name = update_ipaddr(&self.id, &ipstr, &connection);
                match &self.values {
                    Values::Value(msg) => {
                        handle.emit_to("main", "chats", SendMsg{ id: self.id.clone(), name, msg: msg.clone() })?;
                        let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
                        let datetime = now.format("%Y-%m-%d %H:%M:%S");
                        let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", self.id, uid.to_owned(), datetime, "text", msg);
                        connection.lock().unwrap().execute(query).unwrap();
                    }
                    Values::FileData { filename, types, status, contents } => {
                        let mut curpath = std::env::current_exe()?;
                        curpath.pop();
                        curpath.push(filename);
                        match status.as_str() {
                            //"start" => { let _ = std::fs::File::create(&curpath)?; }
                            "start" => {
                                let _ = std::fs::File::create(&curpath)?;
                                unsafe {
                                    FILEDATAS.get_mut().unwrap().insert(curpath, std::collections::VecDeque::new());
                                }
                            }
                            "data" => {
                                unsafe {
                                    FILEDATAS.get_mut().unwrap().get_mut(&curpath).unwrap().push_back(contents.to_vec());
                                }
                                /*
                                let mut file = std::fs::OpenOptions::new().append(true).open(&curpath)?;
                                file.write_all(&contents)?;
                                */
                            }
                            "end" => {
                                let cp = curpath.clone();
                                std::thread::scope(|s| {
                                    s.spawn(move || {
                                        let mut file = std::fs::OpenOptions::new().append(true).open(&cp).unwrap();
                                        unsafe {
                                            while FILEDATAS.get().unwrap().get(&cp).unwrap().len() > 0 {
                                                file.write_all(&FILEDATAS.get_mut().unwrap().get_mut(&cp).unwrap().pop_front().unwrap()).unwrap();
                                            }
                                            FILEDATAS.get_mut().unwrap().remove(&cp);
                                        }
                                        handle.emit_to("main", "userfile", SendFile{ id: self.id.clone(), types: (*types).clone(), name, path: cp.to_string_lossy().to_string() }).unwrap();
                                    });
                                });
                                let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
                                let datetime = now.format("%Y-%m-%d %H:%M:%S");
                                let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, type, chatmsg) VALUES ('{}', '{}', '{}', '{}', '{}');", self.id, uid.to_owned(), datetime, types, curpath.to_string_lossy().to_string());
                                connection.lock().unwrap().execute(query)?;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            "events" => {
                update_ipaddr(&self.id, &ipstr, &connection);
                if let Values::Value(msg) = &self.values {
                    if msg == "closed" {
                        handle.emit_to("main", "exited", self.id.to_string()).unwrap();
                        unsafe {
                            IDS.retain(|item| *item != self.id);
                        }
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }
}
pub static mut FILEDATAS: std::sync::OnceLock<HashMap<PathBuf, std::collections::VecDeque<Vec<u8>>>> = std::sync::OnceLock::new();
#[tauri::command]
pub fn load_finish(handle: tauri::AppHandle, uid: tauri::State<std::sync::Arc<uuid::Uuid>>, socket: tauri::State<std::sync::Arc<std::net::UdpSocket>>) -> () {
    let data = get_admin_info_json(handle, &uid);
    socket.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
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
        let json = AdminInfo {
            name,
            image,
        };
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
fn get_admin_info_json(handle: tauri::AppHandle, uid: &uuid::Uuid) -> String {
    let jsondata = get_admin_info(handle);
    let name;
    if jsondata.is_empty() {
        name = JsonData::new(&uid.to_string(), "name", Values::Value(String::new()));
    } else {
        let jsondata = serde_json::from_str(&jsondata);
        let jsondata = serde_json::from_value::<AdminInfo>(jsondata.unwrap()).unwrap();
        name = JsonData::new(&uid.to_string(), "name", Values::Value(jsondata.name));
    }
    serde_json::to_string(&name).unwrap()
}
pub fn update_ipaddr(id: &str, ip: &str, connect: &std::sync::Arc<std::sync::Mutex<sqlite::Connection>>) -> String {
    let mut name = String::new();
    let query = format!("SELECT * FROM userinfo WHERE userid = '{}';", id);
    connect.lock().unwrap().iterate(query, |pairs| {
        for &(colname, value) in pairs.iter() {
            if !ip.is_empty() {
                if colname == "ip" && value.unwrap() != ip.to_owned() {
                    let query = format!("UPDATE userinfo SET ip = '{}' WHERE userid = '{}';", ip, id);
                    connect.lock().unwrap().execute(query).unwrap();
                }
            }
            if colname == "name" {
                name = value.unwrap().to_string();
            }
        }
        true
    }).unwrap();
    name
}
pub fn init_socket(handle: tauri::AppHandle, connection: std::sync::Arc<std::sync::Mutex<sqlite::Connection>>, uid: std::sync::Arc<uuid::Uuid>, socket: std::sync::Arc<std::net::UdpSocket>) -> std::io::Result<()> {
    loop {
        let mut buf = [0; 3096];
        let (amt, addr) = socket.recv_from(&mut buf)?;
        let jsonvalue = serde_json::from_str(&String::from_utf8_lossy(&buf[..amt]).to_string());
        if jsonvalue.is_err() {
            dbg!("json err:{}", jsonvalue.unwrap());
            continue;
        }
        let jsonvalue = serde_json::from_value::<JsonData>(jsonvalue.unwrap());
        if jsonvalue.is_err() {
            dbg!("JsonData err");
            continue;
        }
        let jsonvalue = jsonvalue?;
        jsonvalue.anaslysis(&addr.to_string(), &addr, &handle, &connection, &uid, &socket).unwrap();
    }
}