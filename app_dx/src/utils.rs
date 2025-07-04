use client::Value;

pub fn add_zero_width_spaces(value: &str) -> String {
    let mut formatted = String::with_capacity(value.len());
    let chars = value.chars();
    let mut prev = None;

    for char in chars {
        if let Some(prev_char) = prev {
            formatted.push(prev_char);

            // Check if the previous character is lowercase and the current character is uppercase
            if prev_char.is_lowercase() && char.is_uppercase() {
                formatted.push('\u{200B}'); // Insert Zero-Width Space
            }

            // Check if the previous character is '<'
            if prev_char == '<' {
                formatted.push('\u{200B}');
            }
        }

        prev = Some(char); // Move to the next character
    }

    // Push the last character if it exists
    if let Some(last_char) = prev {
        formatted.push(last_char);
    }

    formatted
}

pub fn get_array_path(parent_path: &Option<String>, key: &str) -> String {
    format!("{}[{key}]", parent_path.as_ref().unwrap_or(&String::new()))
}

pub fn get_object_path(parent_path: &Option<String>, key: &str) -> String {
    format!("{}.{key}", parent_path.as_ref().unwrap_or(&String::new()))
}

pub fn get_short_type_name(full_path: &str) -> String {
    // Extract the base type name (part before any generics)
    let (base_path, generics) = match full_path.find('<') {
        Some(idx) => (&full_path[..idx], Some(&full_path[idx..])),
        None => (full_path, None),
    };

    // Get the last segment of the base path
    let type_name = base_path.rsplit("::").next().unwrap_or(base_path);

    // If no generics, just return the type name
    match generics {
        None => type_name.to_string(),
        Some(generic_part) => {
            // Process generics recursively
            process_generic_part(type_name, generic_part)
        }
    }
}

pub fn get_type_path_from_ref_value(value: &Value) -> Option<String> {
    value
        .as_object()
        .and_then(|obj| obj.get("type"))
        .and_then(|type_val| type_val.as_object())
        .and_then(|type_obj| type_obj.get("$ref"))
        .and_then(|ref_val| ref_val.as_str())
        .and_then(|ref_str| {
            ref_str
                .strip_prefix("#/$defs/")
                .map(|path| path.to_string())
        })
}

fn process_generic_part(type_name: &str, generic_part: &str) -> String {
    // Ensure the generics start with '<' and find matching '>'
    if !generic_part.starts_with('<') || generic_part.len() < 2 {
        return type_name.to_string();
    }

    // Find matching closing bracket
    let content = match find_matching_bracket(&generic_part[1..]) {
        Some(end_pos) => &generic_part[1..=end_pos],
        None => return type_name.to_string(), // Malformed generics
    };

    // Process the generic arguments
    let processed_args = split_top_level_args(content)
        .iter()
        .map(|arg| get_short_type_name(arg.trim()))
        .collect::<Vec<_>>()
        .join(", ");

    format!("{}<{}>", type_name, processed_args)
}

fn find_matching_bracket(text: &str) -> Option<usize> {
    let mut balance = 0;

    for (i, c) in text.char_indices() {
        match c {
            '<' => balance += 1,
            '>' => {
                if balance == 0 {
                    return Some(i);
                }
                balance -= 1;
            }
            _ => {}
        }
    }

    None // No matching bracket found
}

fn split_top_level_args(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut balance = 0;

    for (i, c) in text.char_indices() {
        match c {
            '<' => balance += 1,
            '>' => balance -= 1,
            ',' if balance == 0 => {
                result.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    // Add the last argument
    if start < text.len() {
        result.push(&text[start..]);
    }

    result
}

pub fn value_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| value.to_string())
}
