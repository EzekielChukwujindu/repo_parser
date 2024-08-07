use crate::code_segmenter::CodeSegmenter;
use tree_sitter::{Parser, Node, Tree, TreeCursor};
use tree_sitter_javascript::language;
use std::fmt::Write;

pub struct JavaScriptSegmenter {
    tree: Tree,
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

    #[allow(dead_code)]
    fn print_node(&self, cursor: &mut TreeCursor, depth: usize) {
        let node = cursor.node();
        let indent = "  ".repeat(depth);
        
        // Print the current node
        println!("{}{}:", indent, node.kind());
        
        // Print node's text if it's a leaf node or has a short text
        let node_text = self.get_node_text(node);
        if node.child_count() == 0 || node_text.lines().count() == 1 {
            println!("{}  Text: \"{}\"", indent, node_text.replace('\n', "\\n"));
        }

        // Print named children
        if node.named_child_count() > 0 {
            println!("{}  Named children:", indent);
            for i in 0..node.named_child_count() {
                if let Some(child) = node.named_child(i) {
                    println!("{}    {}: {}", indent, i, child.kind());
                }
            }
        }

        // Recursively print child nodes
        if cursor.goto_first_child() {
            loop {
                self.print_node(cursor, depth + 1);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    // fn get_node_text(&self, node: Node) -> String {
    //     self.source_code[node.start_byte()..node.end_byte()].to_string()
    // }
    fn process_node(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        match node.kind() {
            "program" => self.process_program(cursor),
            "class_declaration" => self.process_class(cursor),
            "lexical_declaration" => self.process_lexical_declaration(cursor),
            "if_statement" => self.process_if_statement(cursor),
            "await_expression" => self.process_await_expression(cursor),
            "call_expression" => self.process_call_expression(cursor),
            "member_expression" => self.process_member_expression(cursor),
            "function_declaration" => self.process_function(cursor),
            "method_definition" => self.process_function(cursor),
            "function_expression" => self.process_function(cursor),
            "arrow_function" => self.process_arrow_function(cursor),
            "generator_function_declaration" => self.process_function(cursor),
            "generator_function" => self.process_function(cursor),
            "async_function_declaration" => self.process_function(cursor),
            "async_function_expression" => self.process_function(cursor),
            "async_arrow_function" => self.process_arrow_function(cursor),
            "constructor" => self.process_function(cursor),
            "export_statement" => self.process_export(cursor),
            "jsx_element" => self.process_jsx(cursor),
            "variable_declaration" => self.process_variable_declaration(cursor),
            _ => self.get_node_text(node),
        }
    }

    fn process_jsx(&self, _cursor: &mut TreeCursor) -> String {
        "// JSX element\n".to_string()
    }


    fn process_program(&self, cursor: &mut TreeCursor) -> String {
        let mut result = String::new();
        if cursor.goto_first_child() {
            loop {
                result.push_str(&self.process_node(cursor));
                result.push('\n');
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
        result
    }

    fn process_class(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let class_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "UnnamedClass".to_string());
    
        let mut class_def = format!("class {} {{\n", class_name);
    
        if let Some(body) = node.child_by_field_name("body") {
            for child in body.children(&mut body.walk()) {
                // Debug output
                println!("Processing child node of kind: {}", child.kind());
    
                match child.kind() {
                    "method_definition" | "function_declaration" => {
                        let mut child_cursor = child.walk();
                        let method_def = self.process_function(&mut child_cursor);
                        class_def.push_str(&method_def.lines().map(|line| format!("    {}\n", line)).collect::<String>());
                    },
                    _ => {
                        // Handle or skip other child kinds
                        println!("Skipping child node of kind: {}", child.kind());
                    }
                }
            }
        }
    
        class_def.push_str("}\n");
        class_def
    }
    
    fn process_export(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let declaration = node.child(1).unwrap();
        
        if node.child(0).unwrap().kind() == "default" {
            self.process_default_export(declaration)
        } else {
            format!("export {}\n", self.process_node(&mut declaration.walk()))
        }
    }

    fn process_default_export(&self, node: Node) -> String {
        match node.kind() {
            "function_declaration" => {
                let mut cursor = node.walk();
                format!("export default {}", self.process_function(&mut cursor))
            },
            "class_declaration" => {
                let mut cursor = node.walk();
                format!("export default {}", self.process_class(&mut cursor))
            },
            "identifier" => {
                format!("export default {}\n", self.get_node_text(node))
            },
            _ => {
                format!("export default // ...\n")
            }
        }
    }

    fn process_variable_declaration(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let declarator = node.child_by_field_name("declarator").unwrap();
        let name = self.get_node_text(declarator.child_by_field_name("name").unwrap());
        let value = declarator.child_by_field_name("value").unwrap();

        match value.kind() {
            "arrow_function" => {
                let mut child_cursor = value.walk();
                format!("const {} = {}\n", name, self.process_arrow_function(&mut child_cursor))
            },
            "function_expression" => {
                let mut child_cursor = value.walk();
                format!("const {} = {}\n", name, self.process_function(&mut child_cursor))
            },
            _ => format!("const {} = // ...\n", name)
        }
    }

    fn process_lexical_declaration(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();
        
        if let Some(const_node) = node.child_by_field_name("const") {
            result.push_str(&self.get_node_text(const_node));
            result.push(' ');
        }

        if let Some(declarator) = node.child_by_field_name("declarator") {
            if let Some(name) = declarator.child_by_field_name("name") {
                result.push_str(&self.get_node_text(name));
                result.push_str(" = ");
            }
            if let Some(value) = declarator.child_by_field_name("value") {
                let mut value_cursor = value.walk();
                result.push_str(&self.process_node(&mut value_cursor));
            }
        }

        result.push(';');
        result
    }

    fn process_arrow_function(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();

        if let Some(parameters) = node.child_by_field_name("parameters") {
            result.push_str(&self.get_node_text(parameters));
        }

        result.push_str(" => ");

        if let Some(body) = node.child_by_field_name("body") {
            let mut body_cursor = body.walk();
            result.push_str(&self.process_node(&mut body_cursor));
        }

        result
    }

    
    fn process_member_expression(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();

        if let Some(object) = node.child(0) {
            let mut object_cursor = object.walk();
            result.push_str(&self.process_node(&mut object_cursor));
        }

        result.push('.');

        if let Some(property) = node.child(1) {
            result.push_str(&self.get_node_text(property));
        }

        result
    }

    fn process_if_statement(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();

        result.push_str("if (");
        if let Some(condition) = node.child_by_field_name("condition") {
            let mut condition_cursor = condition.walk();
            result.push_str(&self.process_node(&mut condition_cursor));
        }
        result.push_str(") {\n");

        if let Some(consequence) = node.child_by_field_name("consequence") {
            let mut consequence_cursor = consequence.walk();
            result.push_str(&self.process_node(&mut consequence_cursor));
        }

        result.push_str("\n}");

        if let Some(alternative) = node.child_by_field_name("alternative") {
            result.push_str(" else {\n");
            let mut alternative_cursor = alternative.walk();
            result.push_str(&self.process_node(&mut alternative_cursor));
            result.push_str("\n}");
        }

        result
    }

    fn process_await_expression(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();

        result.push_str("await ");
        if let Some(expression) = node.child(1) {
            let mut expression_cursor = expression.walk();
            result.push_str(&self.process_node(&mut expression_cursor));
        }

        result
    }
    fn process_call_expression(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let mut result = String::new();

        if let Some(function) = node.child(0) {
            let mut function_cursor = function.walk();
            result.push_str(&self.process_node(&mut function_cursor));
        }

        result.push('(');
        if let Some(arguments) = node.child_by_field_name("arguments") {
            let mut args_result = Vec::new();
            for i in 0..arguments.named_child_count() {
                if let Some(arg) = arguments.named_child(i) {
                    let mut arg_cursor = arg.walk();
                    args_result.push(self.process_node(&mut arg_cursor));
                }
            }
            result.push_str(&args_result.join(", "));
        }
        result.push(')');

        result
    }

    fn process_function(&self, cursor: &mut TreeCursor) -> String {
        let node = cursor.node();
        let function_name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "".to_string());
    
        let params = node.child_by_field_name("parameters")
            .map(|n| self.get_node_text(n))
            .unwrap_or_else(|| "()".to_string());
    
        format!("function {}{}  {{\n    // implementation\n}}\n", function_name, params)
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
                    let name = self.get_name(node);
                    let start_line = node.start_position().row;

                    writeln!(&mut result, "// Code for: {}", self.get_line(start_line)).unwrap();
                    writeln!(&mut result, "{} {}(...) {{", if node.kind() == "class_declaration" { "class" } else { "function" }, name).unwrap();
                    writeln!(&mut result, "    // implementation").unwrap();
                    writeln!(&mut result, "}}").unwrap();

                    // Skip the children (implementation details)
                    cursor.goto_first_child();
                    while cursor.goto_next_sibling() {}
                    cursor.goto_parent();
                },
                "program" => {
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node(cursor));
                        cursor.goto_parent();
                    }
                },
                _ => {
                    if cursor.goto_first_child() {
                        result.push_str(&self.process_node(cursor));
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

    fn get_name(&self, node: Node) -> &str {
        node.child_by_field_name("name")
            .and_then(|name_node| name_node.utf8_text(self.source_code.as_bytes()).ok())
            .unwrap_or("unknown")
    }

    fn get_line(&self, line_number: usize) -> &str {
        self.source_code.lines().nth(line_number).unwrap_or("")
    }
}
