
use tree_sitter::{Parser, Node, TreeCursor};
use tree_sitter_javascript::language;
use crate::code_segmenter::CodeSegmenter;

pub struct JavaScriptSegmenter {
    tree: tree_sitter::Tree,
    source_code: String,
}

impl CodeSegmenter for JavaScriptSegmenter {
    fn simplify_code(&self) -> String {
        let mut cursor = self.tree.walk();
        self.process_node(&mut cursor)
        // self.print_node(&mut cursor, 0);
        // String::new()
    }

    fn extract_functions_classes(&self) -> String {
        String::new() // Placeholder for potential future implementation
    }
}

impl JavaScriptSegmenter {
    pub fn new(code: String) -> Box<dyn CodeSegmenter> {
        let mut parser = Parser::new();
        parser.set_language(language()).expect("Error loading JavaScript grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse the code");

        Box::new(JavaScriptSegmenter {
            tree,
            source_code: code,
        })
    }

    fn process_node(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        match node.kind() {
            "program" => self.process_program(cursor),
            "class_declaration" => self.process_class(cursor),
            "function_declaration" => self.process_function(cursor),
            "method_definition" => self.process_method(cursor),
            "export_statement" => self.process_export(cursor),
            "comment" => String::new(), // Ignore comments
            _ => self.get_node_text(node),
        }
    }

    fn process_program(&self, cursor: &mut TreeCursor) -> String {
        let mut result = String::new();
        if cursor.goto_first_child() {
            loop {
                let node_text = self.process_node(cursor);
                if !node_text.trim().is_empty() {
                    result.push_str(&node_text);
                    result.push_str("\n");
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
        result.trim_end().to_string()
    }

    fn process_export(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        if let Some(declaration) = node.child(1) {
            match declaration.kind() {
                "function_declaration" | "class_declaration" => {
                    let mut declaration_cursor = declaration.walk();
                    format!("export {}", self.process_node(&mut declaration_cursor))
                },
                "default_keyword" => {
                    if let Some(class_decl) = node.child(2) {
                        let mut class_cursor = class_decl.walk();
                        format!("export default {}", self.process_class(&mut class_cursor))
                    } else {
                        String::new()
                    }
                },
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }

    fn process_class(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let class_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "UnnamedClass".to_string());
        
        let mut class_def = format!("class {} {{\n", class_name);
        
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.children(&mut body.walk()) {
                match child.kind() {
                    "method_definition" => {
                        let mut child_cursor = child.walk();
                        let method_def = self.process_method(&mut child_cursor);
                        class_def.push_str(&method_def.lines().map(|line| format!("    {}\n", line)).collect::<String>());
                    },
                    "public_field_definition" => {
                        let field_def = self.get_node_text(child);
                        class_def.push_str(&format!("    {}\n", field_def));
                    },
                    _ => {}
                }
            }
        }
        
        class_def.push_str("}\n");
        class_def
    }

    fn process_function(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let func_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "unnamed".to_string());
        let params = node.child_by_field_name("parameters")
            .map(|p| self.get_node_text(p))
            .unwrap_or_else(|| "()".to_string());
        
        format!("function {}{}{{ }}\n", func_name, params)
    }

    fn process_method(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let method_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "unnamed".to_string());
        let params = node.child_by_field_name("parameters")
            .map(|p| self.get_node_text(p))
            .unwrap_or_else(|| "()".to_string());
        
        if method_name == "constructor" {
            // Keep the entire constructor method intact
            if let Some(body) = node.child_by_field_name("body") {
                let body_text = self.get_node_text(body);
                format!("{}{}{} {{ }}", method_name, params, body_text)
            } else {
                format!("{}{} {{ }}", method_name, params)
            }
        } else {
            format!("{}{} {{ }}", method_name, params)
        }
    }

    fn get_node_text(&self, node: Node) -> String {
        self.source_code[node.start_byte()..node.end_byte()].to_string()
    }

    #[allow(dead_code)]
    fn process_node_func_class(&self, cursor: &mut TreeCursor) -> String {
        let mut result = String::new();

        loop {
            let node = cursor.node();

            match node.kind() {
                "function_declaration" | "class_declaration" => {
                    // let name = self.get_name(node);
                    let node_text = self.process_node(cursor);
                    result.push_str(&node_text);
                },
                "program" => {
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node_func_class(cursor));
                        cursor.goto_parent();
                    }
                },
                _ => {
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node_func_class(cursor));
                        cursor.goto_parent();
                    }
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        result
    }

    // fn get_name(&self, node: Node) -> String {
    //     node.child_by_field_name("name")
    //         .map(|name_node| self.get_node_text(name_node))
    //         .unwrap_or_else(|| "unnamed".to_string())
    // }
}