mod clipboard;
mod commands;
mod db;
mod error;
mod image;
mod models;
mod scanner;
mod search;
mod tags;
mod thumbnail;

use std::fs;

use commands::AppState;
use db::DATABASE_FILE_NAME;
use error::AppError;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

fn build_app_state(app: &tauri::AppHandle) -> Result<AppState, AppError> {
  let app_data_dir = app.path().app_data_dir().map_err(AppError::PathResolve)?;

  fs::create_dir_all(&app_data_dir).map_err(|source| AppError::DirectoryCreate {
    path: app_data_dir.clone(),
    source,
  })?;

  let database_path = app_data_dir.join(DATABASE_FILE_NAME);
  db::initialize_database(&database_path)?;

  Ok(AppState {
    app_data_dir,
    database_path,
  })
}

fn focus_main_window(app: &tauri::AppHandle) -> Result<(), AppError> {
  if let Some(window) = app.get_webview_window("main") {
    if let Ok(true) = window.is_minimized() {
      let _ = window.unminimize();
    }

    window.show().map_err(AppError::Event)?;
    window.set_focus().map_err(AppError::Event)?;
    window.emit("focus-search-request", ()).map_err(AppError::Event)?;
  }

  Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), AppError> {
  let show = MenuItem::with_id(app, "tray_show", "Show Meme Vault", true, None::<&str>)
    .map_err(AppError::Event)?;
  let quit = MenuItem::with_id(app, "tray_quit", "Quit", true, None::<&str>)
    .map_err(AppError::Event)?;
  let menu = Menu::with_items(app, &[&show, &quit]).map_err(AppError::Event)?;

  let _tray = TrayIconBuilder::with_id("main")
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_menu_event(|app, event| match event.id().as_ref() {
      "tray_show" => {
        let _ = focus_main_window(app);
      }
      "tray_quit" => {
        app.exit(0);
      }
      _ => {}
    })
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        let _ = focus_main_window(tray.app_handle());
      }
    })
    .build(app)
    .map_err(AppError::Event)?;

  Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let global_shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
    .with_shortcut("Ctrl+Shift+Space")
    .expect("invalid global shortcut")
    .with_handler(|app, _shortcut, event| {
      if event.state == ShortcutState::Pressed {
        let _ = focus_main_window(app);
      }
    })
    .build();

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(global_shortcut_plugin)
    .setup(|app| {
      let state =
        build_app_state(app.handle()).map_err(|error| -> Box<dyn std::error::Error> {
          Box::new(error)
        })?;
      app.manage(state);
      setup_tray(app.handle()).map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
      Ok(())
    })
    .on_window_event(|window, event| {
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
      }
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_app_info,
      commands::health_check,
      commands::import_folder,
      commands::get_images,
      commands::get_recent_images,
      commands::get_image_path,
      commands::mark_image_used,
      commands::copy_image_to_clipboard,
      commands::get_tags,
      commands::get_image_tags,
      commands::create_tag,
      commands::delete_tag,
      commands::add_tags_to_images,
      commands::remove_tags_from_images,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}