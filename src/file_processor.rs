use std::collections::HashMap;
use rand::{thread_rng, Rng};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use std::future::Future;
use std::pin::Pin;
use tokio::io::AsyncWriteExt;
use crate::code_segmenter::CodeSegmenter;

async fn process_file(
    file_path: String,
    language_extensions: Arc<HashMap<String, String>>,
    language_segmenters: Arc<HashMap<String, fn(String) -> Box<dyn CodeSegmenter>>>,
    main_root: Arc<String>,
    arch_dir: Arc<PathBuf>,
    summary_file: Arc<tokio::sync::Mutex<tokio::fs::File>>,
) {
    let path = Path::new(&file_path);
    if let Some(extension) = path.extension() {
        if let Some(language) = language_extensions.get(extension.to_str().unwrap()) {
            if let Some(segmenter_fn) = language_segmenters.get(language) {
                match fs::read_to_string(&file_path).await {
                    Ok(code) => {
                        let segmenter = segmenter_fn(code);
                        let simplified_code = segmenter.simplify_code();

                        let main_root_path = Path::new(&*main_root);
                        let save_path = arch_dir.join(path.strip_prefix(main_root_path).unwrap_or(&path));

                        if let Err(e) = fs::create_dir_all(save_path.parent().unwrap()).await {
                            eprintln!("Error creating directory: {}", e);
                            return;
                        }

                        if let Err(e) = fs::write(&save_path, &simplified_code).await {
                            eprintln!("Error writing file {}: {}", save_path.display(), e);
                        }

                        // Add content to summary file
                        let relative_path = path.strip_prefix(main_root_path).unwrap_or(&path);
                        let mut summary = summary_file.lock().await;
                        if let Err(e) = summary.write_all(format!("\n{}\n\n", relative_path.display()).as_bytes()).await {
                            eprintln!("Error writing to summary file: {}", e);
                        }
                        if let Err(e) = summary.write_all(simplified_code.as_bytes()).await {
                            eprintln!("Error writing to summary file: {}", e);
                        }
                        if let Err(e) = summary.write_all(b"\n.................................................................\n").await {
                            eprintln!("Error writing to summary file: {}", e);
                        }
                    }
                    Err(e) => {
                        // Log the error to _arch_xyzxyz/error.txt
                        let error_path = arch_dir.join("error.txt");
                        let error_message = format!("Error reading file {}: {}\n", file_path, e);

                        if let Err(e) = fs::write(&error_path, error_message).await {
                            eprintln!("Error writing to error log: {}", e);
                        }
                    }
                }
            }
        }
    }
}


fn generate_directory_tree<'a>(
    path: &'a Path,
    prefix: &'a str,
    is_last: bool,
) -> Pin<Box<dyn Future<Output = String> + 'a>> {
    Box::pin(async move {
        let mut result = String::new();
        let entry_prefix = if is_last { "└── " } else { "├── " };
        let mut entry = prefix.to_string();
        entry.push_str(entry_prefix);
        entry.push_str(path.file_name().unwrap().to_str().unwrap());
        result.push_str(&entry);
        result.push('\n');

        if path.is_dir() {
            let mut entries = Vec::new();
            if let Ok(mut read_dir) = fs::read_dir(path).await {
                while let Ok(Some(entry)) = read_dir.next_entry().await {
                    let entry_name = entry.file_name();
                    // Convert OsString to &str for easier comparison
                    let entry_name_str = entry_name.to_str().unwrap_or("");

                    // Skip directories that start with '.' or match common directories
                    if entry_name_str.starts_with('.')
                    || entry_name_str == "node_modules"
                    || entry_name_str == "target"
                    || entry_name_str == "dist"
                    || entry_name_str == "build"
                    || entry_name_str == "vendor"
                    || entry_name_str == "__pycache__"
                    || entry_name_str == "logs"
                    || entry_name_str == "coverage"
                    || entry_name_str == "venv"
                    || entry_name_str == "tmp"
                    || entry_name_str == "temp"
                    || entry_name_str == "cache"
                    || entry_name_str == "Pods"
                    || entry_name_str == "DerivedData"
                    || entry_name_str == "bin"
                    || entry_name_str == "pkg"
                    || entry_name_str == "migrations"
                    || entry_name_str == "CMakeFiles"
                    || entry_name_str == "CMakeCache.txt"
                    || entry_name_str == "Gemfile.lock"
                    || entry_name_str == "composer.lock"
                    || entry_name_str == "_build"
                    || entry_name_str == "deps"
                {
                    continue;
                }

                    entries.push(entry);
                }
            }
            entries.sort_by_key(|a| a.file_name());
            let count = entries.len();
            for (i, entry) in entries.into_iter().enumerate() {
                let child_prefix = if is_last { "    " } else { "│   " };
                let child_path = entry.path();
                let child_tree = generate_directory_tree(
                    &child_path,
                    &format!("{}{}", prefix, child_prefix),
                    i == count - 1,
                ).await;
                result.push_str(&child_tree);
            }
        }

        result
    })
}


pub async fn main_parser(
    directory_path: String,
    language_extensions: HashMap<String, String>,
    language_segmenters: HashMap<String, fn(String) -> Box<dyn CodeSegmenter>>,
) {
    let language_extensions = Arc::new(language_extensions);
    let language_segmenters = Arc::new(language_segmenters);
    let main_root = Arc::new(directory_path);

    // Generate random suffix once for the entire process
    let random_suffix: String = thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let main_root_path = Path::new(&*main_root);
    let arch_dir = Arc::new(main_root_path.parent().unwrap().join(format!("_arch_{}", random_suffix)));

    // Create the arch_dir once
    if let Err(e) = fs::create_dir_all(&*arch_dir).await {
        eprintln!("Error creating arch directory: {}", e);
        return;
    }

    // Create and initialize summary file
    let summary_path = arch_dir.join("summary.txt");
    let summary_file = Arc::new(tokio::sync::Mutex::new(
        fs::File::create(&summary_path).await.unwrap()
    ));

    // Generate and write directory tree
    let tree = generate_directory_tree(main_root_path, "", true).await;
    let mut summary = summary_file.lock().await;
    if let Err(e) = summary.write_all(tree.as_bytes()).await {
        eprintln!("Error writing directory tree to summary file: {}", e);
    }
    if let Err(e) = summary.write_all(b"\n.................................................................\n").await {
        eprintln!("Error writing separator to summary file: {}", e);
    }
    drop(summary);  // Release the lock

    let mut tasks = Vec::new();
    let mut stack = vec![PathBuf::from(&*main_root)];

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
                let main_root = Arc::clone(&main_root);
                let arch_dir = Arc::clone(&arch_dir);
                let summary_file = Arc::clone(&summary_file);
                tasks.push(tokio::spawn(async move {
                    process_file(
                        file_path,
                        language_extensions,
                        language_segmenters,
                        main_root,
                        arch_dir,
                        summary_file,
                    ).await;
                }));
            }
        }
    }

    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("Task failed: {}", e);
        }
    }
}