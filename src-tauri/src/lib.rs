mod app;
mod commands;
mod db;
mod duplicates;
mod errors;
mod models;
mod paths;
pub mod scanner;
mod tasks;
pub mod thumbs;
mod viewer;
mod watcher;

use commands::data::{
    backup_database, clear_thumbnail_cache, copy_image_file, create_collection, create_image,
    create_tag, delete_collection_record, delete_image_file, delete_image_record, delete_tag,
    enqueue_thumbnail_generation, export_library_data, get_collection, get_image, get_setting,
    get_settings, get_tag, get_task, get_thumbnail, get_thumbnail_cache_stats, get_viewer_image,
    import_collection, list_collection_tag_assignments, list_collections,
    list_image_tag_assignments, list_images, list_tags, mark_collection_viewed, move_image_file,
    rebuild_index, rename_image_file, restore_database_from_backup, run_duplicate_detection,
    search_library, set_collection_tags, set_image_tags, sync_all_collections, sync_collection,
    update_collection, update_image, update_setting, update_tag,
};
use commands::system::{
    choose_import_folder, copy_path_to_clipboard, copy_text_to_clipboard, get_app_status,
    open_path_in_file_manager,
};
use std::path::Path;
use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Emitter, Manager, Runtime,
};

const MENU_IMPORT_COLLECTION: &str = "import_collection";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .menu(build_menu)
        .on_menu_event(|app, event| {
            if event.id().as_ref() == MENU_IMPORT_COLLECTION {
                let _ = app.emit("menu-import-folder", ());
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let state = app::AppState::initialize(app.handle())?;
            app.asset_protocol_scope()
                .allow_directory(&state.paths().thumbnails_dir, true)?;
            for collection in state.with_db(db::repositories::list_collections)? {
                let collection_path = Path::new(&collection.path);
                if collection_path.is_dir() {
                    app.asset_protocol_scope()
                        .allow_directory(collection_path, true)?;
                }
            }
            app.manage(state);
            watcher::start_file_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            choose_import_folder,
            open_path_in_file_manager,
            copy_text_to_clipboard,
            copy_path_to_clipboard,
            list_collections,
            get_collection,
            create_collection,
            import_collection,
            sync_collection,
            sync_all_collections,
            update_collection,
            mark_collection_viewed,
            delete_collection_record,
            list_images,
            get_image,
            create_image,
            update_image,
            delete_image_record,
            rename_image_file,
            move_image_file,
            copy_image_file,
            delete_image_file,
            list_tags,
            get_tag,
            create_tag,
            update_tag,
            delete_tag,
            list_collection_tag_assignments,
            set_collection_tags,
            list_image_tag_assignments,
            set_image_tags,
            search_library,
            run_duplicate_detection,
            get_settings,
            get_setting,
            update_setting,
            backup_database,
            restore_database_from_backup,
            rebuild_index,
            export_library_data,
            get_thumbnail,
            enqueue_thumbnail_generation,
            get_task,
            get_thumbnail_cache_stats,
            clear_thumbnail_cache,
            get_viewer_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let import = MenuItem::with_id(
        app,
        MENU_IMPORT_COLLECTION,
        "导入文件夹",
        true,
        Some("CmdOrCtrl+O"),
    )?;

    let file = Submenu::with_items(
        app,
        "文件",
        true,
        &[
            &import,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("退出 PhotoView"))?,
        ],
    )?;

    let edit = Submenu::with_items(
        app,
        "编辑",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let window = Submenu::with_items(
        app,
        "窗口",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    let about = AboutMetadata {
        name: Some("PhotoView".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        comments: Some("本地图片查看器与合集管理工具".to_string()),
        ..Default::default()
    };
    let help = Submenu::with_items(
        app,
        "帮助",
        true,
        &[&PredefinedMenuItem::about(
            app,
            Some("关于 PhotoView"),
            Some(about),
        )?],
    )?;

    Menu::with_items(app, &[&file, &edit, &window, &help])
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use serde_json::{json, Value};
    use std::{fs, path::PathBuf};
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        test::{mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY},
        webview::InvokeRequest,
        Webview, WebviewWindowBuilder,
    };

    fn temp_app_data_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("photoview-{name}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("test app data dir should be created");
        dir
    }

    fn create_command_app(app_data_dir: PathBuf) -> tauri::App<MockRuntime> {
        let state = app::AppState::initialize_for_test(app_data_dir)
            .expect("test app state should initialize");

        mock_builder()
            .manage(state)
            .invoke_handler(tauri::generate_handler![
                get_app_status,
                list_collections,
                get_settings,
                update_setting,
            ])
            .build(mock_context(noop_assets()))
            .expect("test app should build")
    }

    fn invoke_json<W: AsRef<Webview<MockRuntime>>>(
        webview: &W,
        cmd: &str,
        body: Value,
    ) -> Result<Value, Value> {
        tauri::test::get_ipc_response(
            webview,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: if cfg!(any(windows, target_os = "android")) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .expect("invoke url should parse"),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|body| {
            body.deserialize::<Value>()
                .expect("command response should be valid json")
        })
    }

    #[test]
    fn tauri_commands_report_status_and_collections() {
        let app_data_dir = temp_app_data_dir("status");
        let app = create_command_app(app_data_dir.clone());
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("test webview should build");

        let status = invoke_json(&webview, "get_app_status", json!({}))
            .expect("status command should succeed");
        assert_eq!(status["product_name"], "PhotoView");
        assert_eq!(status["collection_count"], 0);
        assert_eq!(status["image_count"], 0);

        let collections = invoke_json(&webview, "list_collections", json!({}))
            .expect("collections command should succeed");
        assert_eq!(collections.as_array().map(Vec::len), Some(0));

        drop(webview);
        drop(app);
        fs::remove_dir_all(app_data_dir).expect("test app data dir should be removed");
    }

    #[test]
    fn tauri_commands_update_and_list_settings() {
        let app_data_dir = temp_app_data_dir("settings");
        let app = create_command_app(app_data_dir.clone());
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("test webview should build");

        let updated = invoke_json(
            &webview,
            "update_setting",
            json!({
                "request": {
                    "key": "theme",
                    "value": "dark"
                }
            }),
        )
        .expect("update setting command should succeed");
        assert_eq!(updated["key"], "theme");
        assert_eq!(updated["value"], "dark");

        let settings = invoke_json(&webview, "get_settings", json!({}))
            .expect("settings command should succeed");
        let theme = settings
            .as_array()
            .expect("settings should be an array")
            .iter()
            .find(|setting| setting["key"] == "theme")
            .expect("theme setting should exist");
        assert_eq!(theme["value"], "dark");

        drop(webview);
        drop(app);
        fs::remove_dir_all(app_data_dir).expect("test app data dir should be removed");
    }
}
