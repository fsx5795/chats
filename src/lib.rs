pub type SqlConArc = std::sync::Arc<std::sync::Mutex<sqlite::Connection>>;
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
