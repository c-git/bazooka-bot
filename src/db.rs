use std::{fs, io::Write as _, path::PathBuf};

use tracing::error;

const KEY_VALUE_STORE_FOLDER: &str = "KV";

/// Returns a path if able to be created else logs error and returns `None`
fn get_file_path(key: &str) -> Option<PathBuf> {
    let mut result = PathBuf::from(KEY_VALUE_STORE_FOLDER);
    match std::fs::create_dir_all(&result) {
        Ok(()) => {}
        Err(err_msg) => {
            error!(
                ?err_msg,
                "Failed to create parent directory for key value store: {result:?}"
            );
            return None;
        }
    };
    result = result.join(key);
    if !result.add_extension("json") {
        error!("Unable to add json extension to path: {result:?}");
    }
    Some(result)
}

pub async fn save_kv(key: &str, value: String) {
    let Some(path) = get_file_path(key) else {
        return;
    };

    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
    {
        Ok(file) => file,
        Err(err_msg) => {
            error!(
                ?err_msg,
                "Failed to save content for key: {key} to kv store (file creation failed)"
            );
            return;
        }
    };
    match file.write_all(value.as_bytes()) {
        Ok(_) => {}
        Err(err_msg) => {
            error!(
                ?err_msg,
                "Failed to save content for key: {key} to kv store (write failed)"
            );
        }
    };
}

pub async fn load_kv(key: &str) -> Option<String> {
    let path = get_file_path(key)?;
    match fs::read_to_string(path) {
        Ok(content) => Some(content),
        Err(err_msg) => {
            error!(?err_msg, "Failed to get content for key: {key}");
            None
        }
    }
}
