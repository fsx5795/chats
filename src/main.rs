#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use chats::cmds::{self, Manager};
use log4rs;
fn main() -> () {
    let mut curpath = std::env::current_exe().unwrap();
    curpath.pop();
    //log
    let mut logpath = curpath.clone();
    logpath.push("output.log");
    let logfile = log4rs::append::file::FileAppender::builder()
        .build(logpath)
        .unwrap();
    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            log4rs::config::Root::builder()
                .appender("logfile")
                .build(log::LevelFilter::Info),
        )
        .unwrap();
    log4rs::init_config(config).unwrap();
    //网络文件缓冲区
    unsafe {
        chats::FILEDATAS.get_or_init(|| std::collections::HashMap::new());
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
                    prop.iter()
                        .find(|&(k, _)| k == "id")
                        .map(|(_, v)| uid = std::sync::Arc::new(uuid::Uuid::parse_str(v).unwrap()));
                }
            }
            Err(err) => log::error!("{}", err),
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
        return;
    };
    if let Err(err) = socket.set_multicast_loop_v4(false) {
        log::error!("{}", err);
        return;
    };
    let connect = std::sync::Arc::clone(&connection);
    let id = std::sync::Arc::clone(&uid);
    let udp = std::sync::Arc::clone(&socket);

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            crate::cmds::close_splashscreen,
            crate::cmds::load_finish,
            crate::cmds::get_admin_info,
            crate::cmds::get_user_name,
            crate::cmds::set_admin_info,
            crate::cmds::get_chats_history,
            crate::cmds::send_message,
            crate::cmds::send_file,
            crate::cmds::show_file,
            crate::cmds::close_window
        ])
        .setup(|app| {
            let apphandle = app.app_handle().clone();
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;
            // 创建系统托盘
            let _tray = tauri::tray::TrayIconBuilder::new()
                // 添加托盘图标
                .icon(app.default_window_icon().unwrap().clone())
                // 添加菜单
                .menu(&menu)
                // 禁用鼠标左键点击图标显示托盘菜单
                .menu_on_left_click(false)
                // 监听托盘图标发出的鼠标事件
                .on_tray_icon_event(|tray, event| match event {
                    // 左键点击托盘图标显示窗口
                    tauri::tray::TrayIconEvent::Click {
                        id: _,
                        position: _,
                        rect: _,
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                    } => {
                        let win = tray
                            .app_handle()
                            .get_webview_window("main")
                            .expect("REASON");
                        match win.is_visible() {
                            Ok(visible) if !visible => {
                                win.show().unwrap();
                            }
                            Err(e) => eprintln!("{}", e),
                            _ => (),
                        };
                        // 获取窗口焦点
                        win.set_focus().unwrap();
                    }
                    _ => {}
                })
                // 监听菜单事件
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        let win = app.get_webview_window("main").unwrap();
                        match win.is_visible() {
                            Ok(visible) if !visible => {
                                win.show().unwrap();
                            }
                            Err(e) => eprintln!("{}", e),
                            _ => (),
                        };
                        // 获取窗口焦点
                        win.set_focus().unwrap();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
                std::thread::spawn(move || {
                    if let Err(err) = chats::init_socket(apphandle, connect, id, udp) {
                        log::error!("{}", err)
                    };
                });
                tauri::WebviewWindowBuilder::new(
                    app,
                    "splashscreen",
                    tauri::WebviewUrl::App("splashscreen.html".parse().unwrap()),
                )
                .decorations(false)
                .always_on_top(true)
                .theme(Some(tauri::Theme::Dark))
                .visible(false)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}