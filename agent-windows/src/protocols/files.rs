pub async fn get_file(path: &str) -> (String, i32) {
    match std::fs::read(path) {
        Ok(content) => {
            use base64::{Engine as _, engine::general_purpose};
            let encoded = general_purpose::STANDARD.encode(&content);
            (format!("FILE_CONTENT:{}", encoded), 0)
        }
        Err(e) => (format!("Failed to read file: {}", e), 1),
    }
}

pub async fn upload_file(path: &str, content: &str) -> (String, i32) {
    use base64::{Engine as _, engine::general_purpose};
    let decoded = match general_purpose::STANDARD.decode(content) {
        Ok(data) => data,
        Err(e) => return (format!("Failed to decode content: {}", e), 1),
    };
    
    match std::fs::write(path, decoded) {
        Ok(_) => (format!("File uploaded to: {}", path), 0),
        Err(e) => (format!("Failed to write file: {}", e), 1),
    }
}
