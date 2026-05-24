use crate::{
    app::AppState,
    db::repositories,
    errors::{AppError, AppResult},
    models::{
        CollectionDto, CreateCollectionRequest, CreateImageRequest, CreateTagRequest, ImageDto,
        ImportCollectionRequest, ImportCollectionResult, ListImagesRequest, SettingDto, TagDto,
        ThumbnailDto, UpdateCollectionRequest, UpdateImageRequest, UpdateSettingRequest,
        UpdateTagRequest,
    },
    thumbs::{get_or_create_thumbnail, read_source_metadata, ThumbnailRequest},
};
use tauri::State;

#[tauri::command]
pub fn list_collections(state: State<'_, AppState>) -> AppResult<Vec<CollectionDto>> {
    state.with_db(repositories::list_collections)
}

#[tauri::command]
pub fn get_collection(state: State<'_, AppState>, id: String) -> AppResult<Option<CollectionDto>> {
    state.with_db(|db| repositories::get_collection(db, &id))
}

#[tauri::command]
pub fn create_collection(
    state: State<'_, AppState>,
    request: CreateCollectionRequest,
) -> AppResult<CollectionDto> {
    state.with_db(|db| repositories::create_collection(db, request))
}

#[tauri::command]
pub fn import_collection(
    state: State<'_, AppState>,
    request: ImportCollectionRequest,
) -> AppResult<ImportCollectionResult> {
    state.with_db_mut(|db| repositories::import_collection(db, request))
}

#[tauri::command]
pub fn update_collection(
    state: State<'_, AppState>,
    request: UpdateCollectionRequest,
) -> AppResult<CollectionDto> {
    state.with_db(|db| repositories::update_collection(db, request))
}

#[tauri::command]
pub fn delete_collection_record(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_db(|db| repositories::delete_collection_record(db, &id))
}

#[tauri::command]
pub fn list_images(
    state: State<'_, AppState>,
    request: ListImagesRequest,
) -> AppResult<Vec<ImageDto>> {
    state.with_db(|db| repositories::list_images(db, request))
}

#[tauri::command]
pub fn get_image(state: State<'_, AppState>, id: String) -> AppResult<Option<ImageDto>> {
    state.with_db(|db| repositories::get_image(db, &id))
}

#[tauri::command]
pub fn create_image(
    state: State<'_, AppState>,
    request: CreateImageRequest,
) -> AppResult<ImageDto> {
    state.with_db(|db| repositories::create_image(db, request))
}

#[tauri::command]
pub fn update_image(
    state: State<'_, AppState>,
    request: UpdateImageRequest,
) -> AppResult<ImageDto> {
    state.with_db(|db| repositories::update_image(db, request))
}

#[tauri::command]
pub fn delete_image_record(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_db(|db| repositories::delete_image_record(db, &id))
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> AppResult<Vec<TagDto>> {
    state.with_db(repositories::list_tags)
}

#[tauri::command]
pub fn get_tag(state: State<'_, AppState>, id: String) -> AppResult<Option<TagDto>> {
    state.with_db(|db| repositories::get_tag(db, &id))
}

#[tauri::command]
pub fn create_tag(state: State<'_, AppState>, request: CreateTagRequest) -> AppResult<TagDto> {
    state.with_db(|db| repositories::create_tag(db, request))
}

#[tauri::command]
pub fn update_tag(state: State<'_, AppState>, request: UpdateTagRequest) -> AppResult<TagDto> {
    state.with_db(|db| repositories::update_tag(db, request))
}

#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_db(|db| repositories::delete_tag(db, &id))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Vec<SettingDto>> {
    state.with_db(repositories::list_settings)
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> AppResult<Option<SettingDto>> {
    state.with_db(|db| repositories::get_setting(db, &key))
}

#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    request: UpdateSettingRequest,
) -> AppResult<SettingDto> {
    state.with_db(|db| repositories::update_setting(db, request))
}

#[tauri::command]
pub fn get_thumbnail(
    state: State<'_, AppState>,
    image_id: String,
    target_size: Option<u32>,
) -> AppResult<ThumbnailDto> {
    let image = state
        .with_db(|db| repositories::get_image(db, &image_id))?
        .ok_or_else(|| AppError::new("not_found", "图片不存在"))?;

    let source = read_source_metadata(&image.path)
        .map_err(|value| AppError::new("thumbnail_error", value.to_string()))?;
    let request = ThumbnailRequest::new(
        &image.path,
        &state.paths().thumbnails_dir,
        &image.id,
        source.source_size_bytes,
        source.source_mtime,
        target_size.unwrap_or(192),
    );
    let thumbnail = get_or_create_thumbnail(&request)
        .map_err(|value| AppError::new("thumbnail_error", value.to_string()))?;

    Ok(ThumbnailDto {
        image_id: image.id,
        cache_path: thumbnail.cache_path.display().to_string(),
        url: thumbnail.cache_path.display().to_string(),
        width: thumbnail.width,
        height: thumbnail.height,
        status: match thumbnail.status {
            crate::thumbs::ThumbnailCacheStatus::Hit => "hit".to_string(),
            crate::thumbs::ThumbnailCacheStatus::Miss => "miss".to_string(),
        },
    })
}
