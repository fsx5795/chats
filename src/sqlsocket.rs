use std::{collections::HashMap, io::Write, path::PathBuf};
pub use tauri::Manager;
use crate::{SqlConArc, UdpArc, UidArc};
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AdminInfo {
    pub name: String,
    pub image: String
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
    fn anaslysis(&self, ipstr: &str, addr: &std::net::SocketAddr, handle: &tauri::AppHandle, connection: &SqlConArc, uid: &UidArc, socket: &UdpArc) -> Result<(), Box<dyn std::error::Error>> {
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
                        connection.lock().unwrap().execute(query)?;
                    }
                    if nothas {
                        let query = format!("INSERT INTO userinfo (userid, name, ip) VALUES ('{}', '{}', '{}');", self.id, strval, addr);
                        connection.lock().unwrap().execute(query)?;
                    }
                    unsafe {
                        if !IDS.contains(&self.id) {
                            IDS.push(self.id.clone());
                            let data = get_admin_info_json(handle.clone(), &uid);
                            if let Some(data) = data {
                                socket.send_to(&data.into_bytes(), addr)?;
                            };
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
                        "start" => {
                            let _ = std::fs::File::create(&curpath)?;
                            unsafe {
                                self::FILEDATAS.get_mut().unwrap().insert(curpath, std::collections::VecDeque::new());
                            }
                        }
                        "data" => {
                            unsafe {
                                self::FILEDATAS.get_mut().unwrap().get_mut(&curpath).unwrap().push_back(contents.to_vec());
                            }
                        }
                        "end" => {
                            let cp = curpath.clone();
                            std::thread::scope(|s| {
                                s.spawn(move || {
                                    let mut file = std::fs::OpenOptions::new().append(true).open(&cp).unwrap();
                                    unsafe {
                                        while self::FILEDATAS.get().unwrap().get(&cp).unwrap().len() > 0 {
                                            file.write_all(&self::FILEDATAS.get_mut().unwrap().get_mut(&cp).unwrap().pop_front().unwrap()).unwrap();
                                        }
                                        self::FILEDATAS.get_mut().unwrap().remove(&cp);
                                    }
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
                let name = self::update_ipaddr(&self.id, &ipstr, &connection);
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
                            "start" => {
                                let _ = std::fs::File::create(&curpath)?;
                                unsafe {
                                    self::FILEDATAS.get_mut().unwrap().insert(curpath, std::collections::VecDeque::new());
                                }
                            }
                            "data" => {
                                unsafe {
                                    self::FILEDATAS.get_mut().unwrap().get_mut(&curpath).unwrap().push_back(contents.to_vec());
                                }
                            }
                            "end" => {
                                let cp = curpath.clone();
                                std::thread::scope(|s| {
                                    s.spawn(move || {
                                        let mut file = std::fs::OpenOptions::new().append(true).open(&cp).unwrap();
                                        unsafe {
                                            while self::FILEDATAS.get().unwrap().get(&cp).unwrap().len() > 0 {
                                                file.write_all(&self::FILEDATAS.get_mut().unwrap().get_mut(&cp).unwrap().pop_front().unwrap()).unwrap();
                                            }
                                            self::FILEDATAS.get_mut().unwrap().remove(&cp);
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
                        handle.emit_to("main", "exited", self.id.to_string())?;
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
pub fn get_admin_info_json(handle: tauri::AppHandle, uid: &uuid::Uuid) -> Option<String>{
    let jsondata = crate::cmds::get_admin_info(handle);
    let name;
    if jsondata.is_empty() {
        name = JsonData::new(&uid.to_string(), "name", Values::Value(String::new()));
    } else {
        let jsondata = serde_json::from_str(&jsondata);
        let jsondata = match jsondata {
            Ok(data) => data,
            Err(_) => return None
        };
        let jsondata = match serde_json::from_value::<AdminInfo>(jsondata) {
            Ok(data) => data,
            Err(_) => return None
        };
        name = JsonData::new(&uid.to_string(), "name", Values::Value(jsondata.name));
    }
    match serde_json::to_string(&name) {
        Ok(name) => Some(name),
        Err(_) => None
    }
}
pub fn update_ipaddr(id: &str, ip: &str, connect: &SqlConArc) -> String {
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
pub fn init_socket(handle: tauri::AppHandle, connection: SqlConArc, uid: UidArc, socket: UdpArc) -> std::io::Result<()> {
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