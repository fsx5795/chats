use std::io::Write;
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
    Value(String),
    HeadImg{ status: String, contents: Vec<u8> },
    FileData{ filename: String, status: String, contents: Vec<u8> }
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
    fn anaslysis(&self, ipstr: &str, addr: &std::net::SocketAddr, handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        match self.types.as_str() {
            //联系人上线或修改用户名
            "name" => {
                if let Values::Value(strval) = &self.values {
                    handle.emit_to("main", "ipname", ChatUser{ id: self.id.clone(), name: strval.clone(), })?;
                    match get_db_connection() {
                        Ok(connect) => {
                            let mut b = true;
                            let query = format!("SELECT * FROM userinfo WHERE userid = '{}';", self.id);
                            connect.iterate(query, |_| {
                                b = false;
                                let query = format!("UPDATE userinfo SET userid = '{}', name = '{}', ip = '{}';", self.id, strval, addr);
                                connect.execute(query).unwrap();
                                true
                            }).unwrap();
                            if b {
                                let query = format!("INSERT INTO userinfo (userid, name, ip) VALUES ('{}', '{}', '{}');", self.id, strval, addr);
                                connect.execute(query).unwrap();
                            }
                        }
                        Err(errstr) => {
                            handle.emit_to("main", "error", errstr)?;
                            return Ok(())
                        }
                    };
                    /*
                    unsafe {
                        USERS.insert(self.id.clone(), (addr.to_string(), strval.clone()));
                    }
                    */
                };
            }
            "headimg" => {
                update_ipaddr(&self.id, &ipstr);
                if let Values::HeadImg{status, contents} = &self.values {
                    let mut curpath = std::env::current_exe()?;
                    curpath.pop();
                    curpath.push(&self.id);
                    match status.as_str() {
                        "start" => { let _ = std::fs::File::create(&curpath)?; }
                        "data" => {
                            let mut file = std::fs::OpenOptions::new().append(true).open(&curpath)?;
                            file.write_all(&contents)?;
                        }
                        "end" => {
                            handle.emit_to("main", "userhead", ModifyHead{ id: self.id.clone(), path: self.id.clone() })?;
                            /*
                            let connection = match get_db_connection() {
                                Ok(connect) => connect,
                                Err(errstr) => {
                                    handle.emit_to("main", "error", errstr)?;
                                    return Ok(())
                                }
                            };
                            let now: DateTime<Local> = chrono::Local::now();
                            let datetime = now.format("%Y-%m-%d %H:%M:%S");
                            let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, chatmsg) VALUES ('{}', '{}', '{}', '{}');", self.id, UUID.to_owned(), datetime, curpath.to_string_lossy().to_string());
                            connection.execute(query)?;
                            */
                        }
                        _ => {}
                    }
                }
            }
            "file" => {
                let name = update_ipaddr(&self.id, &ipstr);
                if let Values::FileData { filename, status, contents } = &self.values {
                    let mut curpath = std::env::current_exe()?;
                    curpath.pop();
                    curpath.push(filename);
                    match status.as_str() {
                        "start" => { let _ = std::fs::File::create(&curpath)?; }
                        "data" => {
                            let mut file = std::fs::OpenOptions::new().append(true).open(&curpath)?;
                            file.write_all(&contents)?;
                        }
                        "end" => {
                            println!("{}", curpath.to_string_lossy().to_string());
                            handle.emit_to("main", "userfile", SendFile{ id: self.id.clone(), name, path: curpath.to_string_lossy().to_string() })?;
                            let connection = match get_db_connection() {
                                Ok(connect) => connect,
                                Err(errstr) => {
                                    handle.emit_to("main", "error", errstr)?;
                                    return Ok(())
                                }
                            };
                            let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
                            let datetime = now.format("%Y-%m-%d %H:%M:%S");
                            let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, chatmsg) VALUES ('{}', '{}', '{}', '{}');", self.id, UUID.to_owned(), datetime, curpath.to_string_lossy().to_string());
                            connection.execute(query)?;
                        }
                        _ => {}
                    }
                }
            }
            "chat" => {
                let name = update_ipaddr(&self.id, &ipstr);
                if let Values::Value(strval) = &self.values {
                    handle.emit_to("main", "chats", SendMsg{ id: self.id.clone(), name, msg: strval.clone() })?;
                    let connection = match get_db_connection() {
                        Ok(connect) => connect,
                        Err(errstr) => {
                            handle.emit_to("main", "error", errstr)?;
                            return Ok(())
                        }
                    };
                    let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
                    let datetime = now.format("%Y-%m-%d %H:%M:%S");
                    let query = format!("INSERT INTO chatshistory (uuid, targetId, chattime, chatmsg) VALUES ('{}', '{}', '{}', '{}');", self.id, UUID.to_owned(), datetime, strval);
                    connection.execute(query)?;
                };
            }
            _ => return Err(Box::new(CusErr {
                detail: "未匹配".to_owned(),
            }))
        };
        Ok(())
    }
}
pub static SOCKET: once_cell::sync::Lazy<std::net::UdpSocket> = once_cell::sync::Lazy::new(|| {
    std::net::UdpSocket::bind("0.0.0.0:9527").unwrap()
});
pub static UUID: once_cell::sync::Lazy<uuid::Uuid> = once_cell::sync::Lazy::new(|| {
    let mut inifile = std::env::current_exe().unwrap();
    inifile.pop();
    inifile.push("conf.ini");
    if inifile.exists() {
        let i = ini::Ini::load_from_file(&inifile).unwrap();
        for (_, prop) in &i {
            let res = prop.iter().find(|&(k, _)| k == "id");
            if let Some((_, v)) = res {
                return uuid::Uuid::parse_str(v).unwrap();
            };
        }
    }
    let id = uuid::Uuid::new_v4();
    let mut conf = ini::Ini::new();
    conf.with_section(Some("Admin")).set("id", id);
    conf.write_to_file(inifile).unwrap();
    id
});
#[tauri::command]
pub fn get_admin_info() -> String {
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
    }
    String::new()
}
pub fn update_ipaddr(id: &str, ip: &str) -> String {
    let mut name = String::new();
    if let Ok(connect) = get_db_connection() {
        let query = format!("SELECT * FROM userinfo WHERE userid = '{}';", id);
        connect.iterate(query, |pairs| {
            for &(colname, value) in pairs.iter() {
                if colname == "ip" && value.unwrap() !=  ip.to_owned() {
                    let query = format!("UPDATE userinfo SET userid = '{}', ip = '{}';", id, ip);
                    connect.execute(query).unwrap();
                }
                if colname == "name" {
                    name = value.unwrap().to_string();
                }
            }
            true
        }).unwrap();
    };
    name
}
pub fn get_db_connection() -> Result<sqlite::Connection, String> {
    let mut dbfile = std::env::current_exe().unwrap();
    dbfile.pop();
    dbfile.push("chats.db");
    let dbexists = dbfile.exists();
    if !dbexists {
        if let Err(_) = std::fs::File::create(&dbfile) {
            return Err(String::from("数据库文件生成失败!"));
        }
    }
    let connection = match sqlite::open(dbfile.as_path()) {
        Ok(connect) => connect,
        Err(_) => return Err(String::from("数据库打开失败"))
    };
    let query = "CREATE TABLE IF NOT EXISTS userinfo (userid TEXT, name TEXT, ip TEXT, imgpath VARCHAR(200))";
    match connection.execute(query) {
        Ok(()) => {
            let query = "CREATE TABLE IF NOT EXISTS chatshistory (uuid TEXT, targetId TEXT, chattime DATETIME, chatmsg VARCHAR(200))";
            match connection.execute(query) {
                Ok(()) => Ok(connection),
                Err(_) => Err(String::from("sql语句执行失败!"))
            }
        }
        Err(_) => Err(String::from("sql语句执行失败!"))
    }
}
pub fn init_socket(handle: tauri::AppHandle) -> std::io::Result<()> {
    SOCKET.set_broadcast(true)?;
    SOCKET.set_multicast_loop_v4(true)?;
    let jsondata = get_admin_info();
    let jsondata = serde_json::from_str(&jsondata);
    let jsondata = serde_json::from_value::<AdminInfo>(jsondata.unwrap()).unwrap();
    let name = JsonData::new(&UUID.to_string(), "name", Values::Value(jsondata.name));
    let data = serde_json::to_string(&name)?;
    SOCKET.send_to(&data.into_bytes(), if cfg!(debug_assertions) { "255.255.255.255:8080" } else { "255.255.255.255:9527" })?;
    loop {
        let mut buf = [0; 2048];
        let (amt, addr) = SOCKET.recv_from(&mut buf)?;
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
        let ipstr = addr.to_string();
        let pos = ipstr.find(':').unwrap();
        let ipstr = &ipstr[..pos];
        jsonvalue.anaslysis(ipstr, &addr, &handle).unwrap();
    }
}