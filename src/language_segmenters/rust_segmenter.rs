use std::fmt::Write;
use tree_sitter::{Parser, Node, TreeCursor};
use tree_sitter_rust::language;
use crate::code_segmenter::CodeSegmenter;

pub struct RustSegmenter {
    tree: tree_sitter::Tree,
    source_code: String,
}

impl CodeSegmenter for RustSegmenter {
    fn simplify_code(&self) -> String {
        let mut cursor = self.tree.walk();
        self.process_node(&mut cursor)
    }

    fn extract_functions_classes(&self) -> String {
        let mut cursor = self.tree.walk();
        self.process_node_func_struct(&mut cursor)
    }
}

impl RustSegmenter {
    pub fn new(code: String) -> Box<dyn CodeSegmenter> {
        let mut parser = Parser::new();
        parser.set_language(language()).expect("Error loading Rust grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse Rust code");

        Box::new(RustSegmenter {
            tree,
            source_code: code,
        })
    }

    fn process_node(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        match node.kind() {
            "source_file" => self.process_source_file(cursor),
            "struct_item" => self.process_struct(cursor),
            "function_item" => self.process_function(cursor),
            "impl_item" => self.process_impl(cursor),
            "mod_item" => self.process_mod(cursor),
            _ => self.get_node_text(node),
        }
    }

    fn process_source_file(&self, cursor: &mut TreeCursor) -> String {
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

    fn process_struct(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let struct_text = self.get_node_text(node);
        
        // Split the struct text into lines
        let lines: Vec<&str> = struct_text.lines().collect();
        
        let mut result = String::new();
        let mut in_struct_body = false;
        let mut brace_count = 0;

        for line in lines {
            // Always include lines with attributes or the struct definition
            if line.trim().starts_with('#') || line.contains("struct") {
                result.push_str(line);
                result.push('\n');
                if line.contains('{') {
                    in_struct_body = true;
                    brace_count += 1;
                }
                continue;
            }

            if in_struct_body {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                // Check if this line starts a method
                if line.contains("fn ") {
                    // For methods, we only include the signature
                    let method_signature = line.split('{').next().unwrap_or(line).trim();
                    result.push_str("    ");
                    result.push_str(method_signature);
                    result.push_str(" { ... }\n");
                    
                    // Skip the rest of the method body
                    while brace_count > 1 {
                        brace_count -= 1;
                    }
                } else if !line.trim().starts_with('}') {
                    // Include all other lines in the struct body (fields, etc.)
                    result.push_str(line);
                    result.push('\n');
                }

                if brace_count == 0 {
                    in_struct_body = false;
                    result.push_str("}\n");
                }
            }
        }

        result
    }
    
    fn process_function(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let fn_text = self.get_node_text(node);
        
        // Extract the function signature (everything before the first '{')
        let signature = fn_text.split('{').next().unwrap_or(&fn_text).trim();
        
        format!("{};\n", signature)
    }

    fn process_impl(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let type_name = node.child_by_field_name("type")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "UnknownType".to_string());
        
        let mut impl_block = format!("impl {} {{\n", type_name);
        
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.children(&mut body.walk()) {
                match child.kind() {
                    "function_item" => {
                        let mut child_cursor = child.walk();
                        let method_def = self.process_function(&mut child_cursor);
                        impl_block.push_str(&method_def.lines().map(|line| format!("    {}\n", line)).collect::<String>());
                    },
                    _ => {}
                }
            }
        }
        
        impl_block.push_str("}\n");
        impl_block
    }

    fn process_mod(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mod_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "unnamed_mod".to_string());
        
        format!("mod {};\n", mod_name)
    }


    fn get_node_text(&self, node: Node) -> String {
        self.source_code[node.start_byte()..node.end_byte()].to_string()
    }

    #[allow(dead_code)]
    fn process_node_func_struct(&self, cursor: &mut TreeCursor) -> String {
        let mut result = String::new();

        loop {
            let node = cursor.node();

            match node.kind() {
                "function_item" | "struct_item" | "impl_item" | "mod_item" => {
                    let start_line = node.start_position().row;
                    writeln!(&mut result, "// Code for: {}", self.get_line(start_line)).unwrap();
                    
                    match node.kind() {
                        "function_item" => result.push_str(&self.process_function(cursor)),
                        "struct_item" => result.push_str(&self.process_struct(cursor)),
                        "impl_item" => result.push_str(&self.process_impl(cursor)),
                        "mod_item" => result.push_str(&self.process_mod(cursor)),
                        _ => unreachable!(),
                    }
                },
                "source_file" => {
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node_func_struct(cursor));
                        cursor.goto_parent();
                    }
                },
                _ => {
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node_func_struct(cursor));
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

    fn get_line(&self, line_number: usize) -> &str {
        self.source_code.lines().nth(line_number).unwrap_or("")
    }
}