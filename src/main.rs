mod code_segmenter;
mod language_segmenters;
mod file_processor;

use std::collections::HashMap;
use tokio;

use crate::code_segmenter::CodeSegmenter;
use crate::language_segmenters::*;
use crate::file_processor::main_parser;

#[tokio::main]
async fn main() {
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

    let directory_path = String::from("C:\\Users\\ezeki\\OneDrive\\Documents\\IT_Projects\\Software_architect\\New_Architect\\Figraph\\Figraph_5.0\\figraph-backend\\figraph\\arch_parser");
    main_parser(directory_path, language_extensions, language_segmenters).await;
}
