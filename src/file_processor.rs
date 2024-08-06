use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use crate::code_segmenter::CodeSegmenter;

async fn process_file(
    file_path: String,
    language_extensions: Arc<HashMap<String, String>>,
    language_segmenters: Arc<HashMap<String, fn(String) -> Box<dyn CodeSegmenter>>>,
    main_root: Arc<String>,
) {
    let path = Path::new(&file_path);
    if let Some(extension) = path.extension() {
        if let Some(language) = language_extensions.get(extension.to_str().unwrap()) {
            if let Some(segmenter_fn) = language_segmenters.get(language) {
                match fs::read_to_string(&file_path).await {
                    Ok(code) => {
                        let segmenter = segmenter_fn(code);
                        let simplified_code = segmenter.simplify_code();

                        // Convert Arc<String> to &str and then to Path
                        let main_root_path = Path::new(&*main_root); // Dereference Arc to &str
                        let arch_dir = main_root_path.parent().unwrap().join("_arch_");
                        let save_path = arch_dir.join(path.strip_prefix(main_root_path).unwrap_or(&path));

                        if let Err(e) = fs::create_dir_all(save_path.parent().unwrap()).await {
                            eprintln!("Error creating directory: {}", e);
                            return;
                        }

                        if let Err(e) = fs::write(&save_path, simplified_code).await {
                            eprintln!("Error writing file {}: {}", save_path.display(), e);
                        }
                    }
                    Err(e) => {
                        // Log the error to _arch_/error.txt
                        let error_path = Path::new(&*main_root).parent().unwrap().join("_arch_").join("error.txt");
                        let error_message = format!("Error reading file {}: {}\n", file_path, e);

                        if let Err(e) = fs::create_dir_all(error_path.parent().unwrap()).await {
                            eprintln!("Error creating directory for error log: {}", e);
                            return;
                        }

                        if let Err(e) = fs::write(&error_path, error_message).await {
                            eprintln!("Error writing to error log: {}", e);
                        }
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
    let main_root = Arc::new(directory_path);

    let mut tasks = Vec::new();
    let mut stack = vec![PathBuf::from(&*main_root)];  // Dereference Arc to PathBuf

    while let Some(current_dir) = stack.pop() {
        let mut entries = fs::read_dir(&current_dir).await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let file_path = path.to_str().unwrap().to_string();
                let language_extensions = Arc::clone(&language_extensions);
                let language_segmenters = Arc::clone(&language_segmenters);
                let main_root = Arc::clone(&main_root);  // Clone the Arc
                tasks.push(tokio::spawn(process_file(
                    file_path,
                    language_extensions,
                    language_segmenters,
                    main_root,
                )));
            }
        }
    }

    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("Task failed: {}", e);
        }
    }
}
