use crate::sqlsocket::Manager;
#[tauri::command]
pub fn load_finish(handle: tauri::AppHandle, uid: tauri::State<crate::UidArc>, socket: tauri::State<crate::UdpArc>) -> () {
    let data = crate::sqlsocket::get_admin_info_json(handle, &uid);
    if let Some(data) = data {
        socket.send_to(&data.into_bytes(), "234.0.0.0:9527").unwrap();
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
        let json = crate::sqlsocket::AdminInfo {
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