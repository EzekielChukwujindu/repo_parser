use std::fmt::Write;
use tree_sitter::{Parser, Node, TreeCursor};
use tree_sitter_python::language;
use crate::code_segmenter::CodeSegmenter;

pub struct PythonSegmenter {
    tree: tree_sitter::Tree,
    source_code: String,
}

impl CodeSegmenter for PythonSegmenter {
    fn simplify_code(&self) -> String {
        let mut cursor = self.tree.walk();
        self.process_node(&mut cursor)
    }

    fn extract_functions_classes(&self) -> Vec<String> {
        Vec::new()
    }
}

impl PythonSegmenter {
    pub fn new(code: String) -> Box<dyn CodeSegmenter> {
        let mut parser = Parser::new();
        parser.set_language(language()).expect("Error loading Python grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse Python code");

        Box::new(PythonSegmenter {
            tree,
            source_code: code,
        })
    }

    fn process_node(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        match node.kind() {
            "module" => self.process_module(cursor),
            "class_definition" => self.process_class(cursor),
            "function_definition" => self.process_function(cursor),
            "async_function_definition" => self.process_function(cursor),
            "decorated_definition" => self.process_decorated_definition(cursor),
            _ => self.get_node_text(node),
        }
    }

    fn process_module(&self, cursor: &mut TreeCursor) -> String {
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

    fn process_decorated_definition(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();
    
        // Process decorators
        for child in node.children(&mut node.walk()) {
            if child.kind() == "decorator" {
                let decorator_text = self.get_node_text(child);
                result.push_str(&decorator_text);
                result.push('\n');
            }
        }
    
        // Process the decorated definition (function or class)
        if let Some(definition) = node.child(node.child_count() - 1) {
            let mut subcursor = definition.walk();
            result.push_str(&self.process_node(&mut subcursor));
        }
    
        result
    }

    fn process_class(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let class_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "UnnamedClass".to_string());
        
        let mut class_def = format!("class {}:\n", class_name);
        
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.children(&mut body.walk()) {
                match child.kind() {
                    "function_definition" | "async_function_definition" => {
                        let mut child_cursor = child.walk();
                        let method_def = self.process_function(&mut child_cursor);
                        class_def.push_str(&method_def.lines().map(|line| format!("    {}\n", line)).collect::<String>());
                    },
                    "decorated_definition" => {
                        let mut child_cursor = child.walk();
                        let decorated_def = self.process_decorated_definition(&mut child_cursor);
                        class_def.push_str(&decorated_def.lines().map(|line| format!("    {}\n", line)).collect::<String>());
                    },
                    _ => {}
                }
            }
        }
        
        if !class_def.ends_with("\n") {
            class_def.push('\n');
        }
        
        class_def
    }
    fn process_function(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let is_async = node.children(&mut node.walk()).any(|child| child.kind() == "async");
        let func_name = self.get_node_text(node.child_by_field_name("name").unwrap());
        let params = self.get_node_text(node.child_by_field_name("parameters").unwrap());
        
        let mut func_def = if is_async {
            format!("async def {}{}:\n", func_name, params)
        } else {
            format!("def {}{}:\n", func_name, params)
        };

        if func_name == "__init__" {
            // Keep the entire __init__ method intact
            if let Some(body) = node.child_by_field_name("body") {
                let body_text = self.get_node_text(body);
                for line in body_text.lines() { 
                    writeln!(func_def, "    {}", line).unwrap();
                }
            }
        } else {
            func_def.push_str("    pass\n");
        }

        func_def
    }

    fn get_node_text(&self, node: Node) -> String {
        self.source_code[node.start_byte()..node.end_byte()].to_string()
    }
}