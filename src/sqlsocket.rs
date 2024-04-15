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
pub fn init_socket(handle: tauri::AppHandle, connection: chats::SqlConArc, uid: chats::UidArc, socket: chats::UdpArc) -> std::io::Result<()> {
    loop {
        let mut buf = [0; 3096];
        let (amt, addr) = socket.recv_from(&mut buf)?;
        let jsonvalue = serde_json::from_str(&String::from_utf8_lossy(&buf[..amt]).to_string());
        if jsonvalue.is_err() {
            dbg!("json err:{}", jsonvalue.unwrap());
            continue;
        }
        let jsonvalue = serde_json::from_value::<chats::JsonData>(jsonvalue.unwrap());
        if jsonvalue.is_err() {
            dbg!("JsonData err");
            continue;
        }
        let jsonvalue = jsonvalue?;
        jsonvalue.anaslysis(&addr.to_string(), &addr, &handle, &connection, &uid, &socket).unwrap();
    }
}