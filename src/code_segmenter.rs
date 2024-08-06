pub(crate) trait CodeSegmenter: Send {
    fn simplify_code(&self) -> String;
    #[allow(dead_code)]
    fn extract_functions_classes(&self) -> Vec<String>;
}

