mod code_segmenter;
mod language_segmenters;
mod file_processor;

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use tokio;

use crate::code_segmenter::CodeSegmenter;
use crate::language_segmenters::*;
use crate::file_processor::main_parser;
use git2::Repository;

fn normalize_path(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(path) => path,
        Err(_) => path.to_path_buf(), // Fallback to the original path if canonicalization fails
    }
}

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <git-repo-url|directory-path>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];

    // Remove the '--' prefix if present
    let input_path = input_path.trim_start_matches("--");
    
    // Define the temp directory path based on the directory containing Cargo.toml
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let mut temp_dir = PathBuf::from(cargo_manifest_dir);
    temp_dir.push("temp/temp");

    // Normalize the input path and handle cloning or using directly
    let directory_path = if input_path.starts_with("http://") || input_path.starts_with("https://") {
        // Clone the git repository into the temp directory
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();
        match Repository::clone(input_path, &temp_dir) {
            Ok(_) => normalize_path(&temp_dir),
            Err(e) => {
                eprintln!("Failed to clone the repository: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Normalize and use the directory path directly
        normalize_path(Path::new(input_path))
        
    };

    // Define language extensions and segmenters
    let language_extensions: HashMap<String, String> = [
        ("py", "python"),
        ("js", "js"),
        ("cobol", "cobol"),
        ("c", "c"),
        ("cpp", "cpp"),
        ("cs", "csharp"),
        ("rb", "ruby"),
        ("scala", "scala"),
        ("rs", "rust"),
        ("go", "go"),
        ("kt", "kotlin"),
        ("lua", "lua"),
        ("pl", "perl"),
        ("ts", "ts"),
        ("java", "java"),
        ("php", "php"),
        ("ex", "elixir"),
        ("exs", "elixir"),
    ].iter().map(|&(k, v)| (k.to_string(), v.to_string())).collect();

    let language_segmenters: HashMap<String, fn(String) -> Box<dyn CodeSegmenter>> = [
        ("python".to_string(), PythonSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        ("js".to_string(), JavaScriptSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // Uncomment and add other segmenters as needed
        // ("cobol".to_string(), CobolSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("c".to_string(), CSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("cpp".to_string(), CppSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("csharp".to_string(), CSharpSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("ruby".to_string(), RubySegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("scala".to_string(), ScalaSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("rust".to_string(), RustSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("go".to_string(), GoSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("kotlin".to_string(), KotlinSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("lua".to_string(), LuaSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("perl".to_string(), PerlSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("ts".to_string(), TypeScriptSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("java".to_string(), JavaSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("php".to_string(), PhpSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
        // ("elixir".to_string(), ElixirSegmenter::new as fn(String) -> Box<dyn CodeSegmenter>),
    ].iter().cloned().collect();

    // Convert directory path to string and pass to main_parser
    let directory_path_str = directory_path.to_str().unwrap().to_string();
    main_parser(directory_path_str, language_extensions, language_segmenters).await;
}
