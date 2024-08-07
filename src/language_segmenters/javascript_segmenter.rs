use crate::code_segmenter::CodeSegmenter;
use tree_sitter::{Parser, Node, Tree};
use tree_sitter_javascript::language;

pub struct JavaScriptSegmenter {
    code: String,
    tree: Tree,
}

impl JavaScriptSegmenter {
    pub fn new(code: String) -> Box<dyn CodeSegmenter> {
        let mut parser = Parser::new();
        parser.set_language(language()).expect("Error loading JavaScript grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse the code");

        Box::new(JavaScriptSegmenter { code, tree })
    }

    fn process_node(&self, node: &Node, inside_class: bool) -> String {
        match node.kind() {
            "class_declaration" => self.process_class(node),
            "function_declaration" | "method_definition" => self.process_function(node, inside_class),
            _ => format!("{:?}", node), // Simplified; handle more cases as needed
        }
    }

    fn process_class(&self, node: &Node) -> String {
        let class_name = node.child_by_field_name("name")
            .map(|name_node| self.code[name_node.start_byte()..name_node.end_byte()].to_string())
            .unwrap_or_default();

        let body = node.children(&mut node.walk())
            .filter(|child| child.kind() == "method_definition")
            .map(|child| self.process_node(&child, true))
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join("\n");

        format!("class {} {{\n{}\n}}", class_name, body.lines().map(|line| format!("    {}", line)).collect::<Vec<String>>().join("\n"))
    }

    fn process_function(&self, node: &Node, inside_class: bool) -> String {
        let function_name = node.child_by_field_name("name")
            .map(|name_node| self.code[name_node.start_byte()..name_node.end_byte()].to_string())
            .unwrap_or_default();

        let params = node.child_by_field_name("parameters")
            .map(|params_node| {
                params_node.children(&mut params_node.walk())
                    .filter(|child| child.kind() == "identifier")
                    .map(|child| self.code[child.start_byte()..child.end_byte()].to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            })
            .unwrap_or_default();

        if inside_class && function_name == "constructor" {
            return format!("constructor({}) {{ ... }}", params);
        }

        format!("{}({}) {{ ... }}", function_name, params)
    }
}

// Implement the CodeSegmenter trait
impl CodeSegmenter for JavaScriptSegmenter {
    fn simplify_code(&self) -> String {
        // Simplify code by processing the whole tree
        let mut result = String::new();
        let root_node = self.tree.root_node();
        for node in root_node.children(&mut root_node.walk()) {
            result.push_str(&self.process_node(&node, false));
            result.push('\n');
        }
        result
    }

    fn extract_functions_classes(&self) -> String {
        // Extract functions and classes from the tree
        let mut result = String::new();
        let root_node = self.tree.root_node();
        for node in root_node.children(&mut root_node.walk()) {
            let processed = self.process_node(&node, false);
            if !processed.is_empty() {
                result.push_str(&processed);
            }
        }
        result
    }
}
