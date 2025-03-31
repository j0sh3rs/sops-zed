use std::{fs, process};
use std::collections::HashMap;
use std::sync::Mutex;
use zed_extension_api::{self as zed, Result};
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::ffi::c_char;

// Use a global static to track state between hook callbacks
static DECRYPTED_FILES: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

struct SopsExtension;

impl SopsExtension {
    fn is_sops_encrypted(content: &str) -> bool {
        // Check for SOPS header markers
        content.contains("sops:") &&
        (content.contains("encrypted_") || content.contains("ENC["))
    }

    fn decrypt_file(path: &str) -> Result<String> {
        let output = process::Command::new("sops")
            .arg("-d")
            .arg(path)
            .output()
            .map_err(|e| format!("Failed to execute sops: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "sops decryption failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn encrypt_file(path: &str, content: &str) -> Result<()> {
        // Write decrypted content to a temporary file
        let temp_file = format!("{}.tmp", path);
        fs::write(&temp_file, content)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // Encrypt the temp file and redirect output to the original file
        let output = process::Command::new("sops")
            .arg("-e")
            .arg("--in-place")
            .arg(&temp_file)
            .output()
            .map_err(|e| format!("Failed to execute sops: {}", e))?;

        if !output.status.success() {
            fs::remove_file(&temp_file).ok();
            return Err(format!(
                "sops encryption failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Move the encrypted temp file back to the original path
        fs::rename(&temp_file, path)
            .map_err(|e| format!("Failed to move encrypted file: {}", e))?;

        Ok(())
    }
}

impl zed::Extension for SopsExtension {
    fn new() -> Self {
        Self
    }
}

// Implement document callbacks
#[unsafe(no_mangle)]
pub extern "C" fn open_document(_buffer_ptr: *mut u8, path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Convert path to string
    let path_str = unsafe {
        CStr::from_ptr(path).to_string_lossy().to_string()
    };

    // Read the file content
    if let Ok(content) = fs::read_to_string(&path_str) {
        if SopsExtension::is_sops_encrypted(&content) {
            match SopsExtension::decrypt_file(&path_str) {
                Ok(decrypted) => {
                    // Store original content for later encryption
                    let mut files = DECRYPTED_FILES.lock().unwrap();
                    files.insert(path_str.clone(), content);

                    // Write decrypted content to file
                    if let Err(e) = fs::write(&path_str, decrypted) {
                        eprintln!("Failed to write decrypted content: {}", e);
                        return false;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to decrypt SOPS file: {}", e);
                    return false;
                }
            }
        }
    } else {
        return false;
    }

    true
}

#[unsafe(no_mangle)]
pub extern "C" fn save_document(_buffer_ptr: *mut u8, path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Convert path to string
    let path_str = unsafe {
        CStr::from_ptr(path).to_string_lossy().to_string()
    };

    let files = DECRYPTED_FILES.lock().unwrap();
    if files.contains_key(&path_str) {
        // The file content will be read from disk and encrypted
        if let Ok(content) = fs::read_to_string(&path_str) {
            // Drop the lock to avoid deadlock in encrypt_file
            drop(files);

            // Encrypt the file contents
            if let Err(e) = SopsExtension::encrypt_file(&path_str, &content) {
                eprintln!("Failed to encrypt SOPS file: {}", e);
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

#[unsafe(no_mangle)]
pub extern "C" fn close_document(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Convert path to string
    let path_str = unsafe {
        CStr::from_ptr(path).to_string_lossy().to_string()
    };

    // Check if this file was decrypted
    let files = DECRYPTED_FILES.lock().unwrap();
    if let Some(original_content) = files.get(&path_str) {
        // Restore the original encrypted content
        if let Err(e) = fs::write(&path_str, original_content) {
            eprintln!("Failed to restore encrypted content: {}", e);
            return false;
        }
    }

    // Remove from our tracking
    let mut files = DECRYPTED_FILES.lock().unwrap();
    files.remove(&path_str);

    true
}

zed::register_extension!(SopsExtension);
