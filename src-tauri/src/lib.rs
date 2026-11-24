mod secret_store;
mod status_watch;
mod ai_hub;
mod chatgpt;
mod commands;
mod computer_session;
mod db;
mod hub_commands;
mod windows;
mod browser_auth;
mod google_oauth;
mod profiles;
use commands::{launch_product, list_products, shutdown_app};
use db::HostDatabase;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use windows::{WindowKind, WindowManager};

pub struct AppState {
    profiles: Mutex<profiles::ProfileService>,
    database: HostDatabase,
    hub: Mutex<ai_hub::AiHub>,
    windows: WindowManager,
    chatgpt: chatgpt::ChatgptService,
}
fn report(app: &tauri::AppHandle, result: Result<(), String>) {
    if let Err(error) = result {
        if let Ok(mut hub) = app.state::<AppState>().hub.lock() {
            hub.startup_message(error);
        }
        let _ = app.emit("hub-changed", ());
        let _ = app.state::<AppState>().windows.show(app, WindowKind::Main);
    }
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app,args,_| {
            // A second invocation retains typed target dispatch without starting another hub.
            report(app, commands::launch_from_arguments(app.state::<AppState>().inner(), args.into_iter().skip(1)));
            report(app, app.state::<AppState>().windows.show(app,WindowKind::Main));
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            std::fs::create_dir_all(&root)?;
            app.manage(AppState { profiles: Mutex::new(profiles::ProfileService::new(&root).map_err(std::io::Error::other)?), database: HostDatabase::new(root.join("overlay-forge-host.sqlite3"))?, hub: Mutex::new(ai_hub::AiHub::default()), windows: WindowManager::default(), chatgpt: chatgpt::ChatgptService::new().map_err(std::io::Error::other)? });
            let state = app.state::<AppState>();
            status_watch::start(app.handle().clone());
            report(app.handle(),state.chatgpt.restore(&state.database));
            state.windows.preload(app.handle()).map_err(std::io::Error::other)?;
            report(app.handle(), app.global_shortcut().on_shortcut("Ctrl+Alt+Space", |app,_,event| {
                if event.state == ShortcutState::Pressed { report(app,app.state::<AppState>().windows.show(app,WindowKind::QuickPrompt)); }
            }).map_err(|e|format!("Ctrl+Alt+Space unavailable: {e}. Use Open quick prompt in the main app.")));
            let open = MenuItem::with_id(app,"open","Open Overlay Forge",true,None::<&str>)?;
            let quick = MenuItem::with_id(app,"quick","Quick prompt",true,None::<&str>)?;
            let quit = MenuItem::with_id(app,"quit","Quit Overlay Forge",true,None::<&str>)?;
            let menu = Menu::with_items(app,&[&open,&quick,&quit])?;
            let mut tray = TrayIconBuilder::new().tooltip("Overlay Forge — resident AI hub").menu(&menu).on_menu_event(|app,event| {
                let state = app.state::<AppState>();
                match event.id.as_ref() {
                    "open" => report(app,state.windows.show(app,WindowKind::Main)),
                    "quick" => report(app,state.windows.show(app,WindowKind::QuickPrompt)),
                    "quit" => report(app,state.windows.quit(app)),
                    _ => (),
                }
            });
            if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
            tray.build(app)?;
            state.windows.show(app.handle(),WindowKind::Main).map_err(std::io::Error::other)?;
            Ok(())
        })
        .on_window_event(|window,event| {
            if let tauri::WindowEvent::CloseRequested { api,.. } = event {
                api.prevent_close();
                let app = window.app_handle();
                let kind = if window.label() == "quick-prompt" { WindowKind::QuickPrompt } else { WindowKind::Main };
                report(app,app.state::<AppState>().windows.hide(app,kind));
            }
        })
        .invoke_handler(tauri::generate_handler![profiles::identity_snapshot,profiles::identity_sign_in,profiles::identity_cancel,profiles::identity_sign_out,list_products,launch_product,shutdown_app,hub_commands::hub_snapshot,hub_commands::hub_draft,hub_commands::hub_submit,hub_commands::hub_action,hub_commands::hub_stop_all,chatgpt::chatgpt_sign_in,chatgpt::chatgpt_verify,chatgpt::chatgpt_disconnect,windows::show_main,windows::show_quick_prompt,windows::hide_quick_prompt])
        .run(tauri::generate_context!())
        .expect("error while running Overlay Forge");
}
