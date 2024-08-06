use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::task;
use crate::code_segmenter::CodeSegmenter;

async fn process_file(
    file_path: String,
    language_extensions: Arc<HashMap<String, String>>,
    language_segmenters: Arc<HashMap<String, fn(String) -> Box<dyn CodeSegmenter>>>,
    main_root: String,
) {
    let path = Path::new(&file_path);
    if let Some(extension) = path.extension() {
        if let Some(language) = language_extensions.get(extension.to_str().unwrap()) {
            if let Some(segmenter_fn) = language_segmenters.get(language) {
                match fs::read_to_string(&file_path).await {
                    Ok(code) => {
                        let segmenter = segmenter_fn(code);
                        let simplified_code = segmenter.simplify_code();

                        let save_path = Path::new(&main_root).join("_arch_").join(path.strip_prefix(&main_root).unwrap());
                        if let Err(e) = fs::create_dir_all(save_path.parent().unwrap()).await {
                            eprintln!("Error creating directory: {}", e);
                            return;
                        }

                        if let Err(e) = fs::write(&save_path, simplified_code).await {
                            eprintln!("Error writing file {}: {}", save_path.display(), e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading file {}: {}", file_path, e);
                    }
                }
            }
        }
    }
}

pub async fn main_parser(
    directory_path: String,
    language_extensions: HashMap<String, String>,
    language_segmenters: HashMap<String, fn(String) -> Box<dyn CodeSegmenter>>,
) {
    let language_extensions = Arc::new(language_extensions);
    let language_segmenters = Arc::new(language_segmenters);
    let mut tasks = Vec::new();

    let mut entries = fs::read_dir(&directory_path).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let file_path = entry.path();
        if file_path.is_file() {
            let task = task::spawn(process_file(
                file_path.to_str().unwrap().to_string(),
                Arc::clone(&language_extensions),
                Arc::clone(&language_segmenters),
                directory_path.clone(),
            ));
            tasks.push(task);
        }
    }

    for task in tasks {
        task.await.unwrap();
    }
}