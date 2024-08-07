use tree_sitter::{Parser, Node, TreeCursor};
use std::fmt::Write;
use tree_sitter_java::language;
use crate::code_segmenter::CodeSegmenter;

pub struct JavaSegmenter {
    tree: tree_sitter::Tree,
    source_code: String,
}

impl CodeSegmenter for JavaSegmenter {
    fn simplify_code(&self) -> String {
        let mut cursor = self.tree.walk();
        self.process_node(&mut cursor)
    }

    fn extract_functions_classes(&self) -> String {
        let mut cursor = self.tree.walk();
        self.process_node_func_class(&mut cursor)
    }
}

impl JavaSegmenter {
    pub fn new(code: String) -> Box<dyn CodeSegmenter> {
        let mut parser = Parser::new();
        parser.set_language(language())
            .expect("Error loading Java grammar");
        let tree = parser.parse(&code, None)
            .expect("Failed to parse Java code");

        Box::new(JavaSegmenter {
            tree,
            source_code: code,
        })
    }

    fn process_node(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        match node.kind() {
            "program" => self.process_program(cursor),
            "package_declaration" => self.get_node_text(node),
            "import_declaration" => self.get_node_text(node),
            "class_declaration" => self.process_class(cursor),
            "method_declaration" => self.process_method(cursor),
            "constructor_declaration" => self.process_constructor(cursor),
            "field_declaration" => self.get_node_text(node),
            _ => String::new(),
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

    fn process_class(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let class_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "UnnamedClass".to_string());
        
        let mut class_def = format!("class {} {{\n", class_name);
        
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.named_children(&mut body.walk()) {
                match child.kind() {
                    "method_declaration" => {
                        let mut child_cursor = child.walk();
                        let method_def = self.process_node(&mut child_cursor);
                        class_def.push_str(&method_def.lines().map(|line| format!("    {}\n", line)).collect::<String>());
                    },
                    "constructor_declaration" => {
                        let mut child_cursor = child.walk();
                        let constructor_def = self.process_constructor(&mut child_cursor);
                        class_def.push_str(&constructor_def);
                    },
                    "field_declaration" => {
                        class_def.push_str(&self.get_node_text(child));
                        class_def.push('\n');
                    },
                    _ => {}
                }
            }
        }
        
        class_def.push_str("}\n");
        class_def
    }

    fn process_method(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let method_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "unnamedMethod".to_string());
        let return_type = node.child_by_field_name("type")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "void".to_string());
        let parameters = node.child_by_field_name("parameters")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "()".to_string());
        
        format!("{} {}{};\n", return_type, method_name, parameters)
    }

    fn process_constructor(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let constructor_text = self.get_node_text(node);
        format!("    // Constructor implementation\n {} ", constructor_text)
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
                "method_declaration" | "constructor_declaration" => {
                    // let name = self.get_name(node);
                    let start_line = node.start_position().row;
                    let end_line = node.end_position().row;

                    writeln!(&mut result, "// Code for: {}", self.get_line(start_line)).unwrap();
                    writeln!(&mut result, "{} {{ ... }}", self.get_signature(node)).unwrap();
                    writeln!(&mut result, "// End: {}", self.get_line(end_line)).unwrap();
                    writeln!(&mut result).unwrap();

                    // Skip the children (implementation details)
                    cursor.goto_first_child();
                    while cursor.goto_next_sibling() {}
                    cursor.goto_parent();
                },
                "class_declaration" => {
                    let class_text = self.get_node_text(node.child(0).unwrap_or(node));
                    writeln!(&mut result, "{} {{", class_text).unwrap();
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node_func_class(cursor));
                        cursor.goto_parent();
                    }
                    writeln!(&mut result, "}}").unwrap();
                },
                "program" | "class_body" => {
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

    // fn get_name(&self, node: Node) -> &str {
    //     node.child_by_field_name("name")
    //         .and_then(|name_node| name_node.utf8_text(self.source_code.as_bytes()).ok())
    //         .unwrap_or("unknown")
    // }

    fn get_line(&self, line_number: usize) -> &str {
        self.source_code.lines().nth(line_number).unwrap_or("")
    }

    fn get_signature(&self, node: Node) -> String {
        self.get_node_text(node.named_child(0).unwrap_or(node))
    }
}