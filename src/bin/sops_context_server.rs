// src/bin/sops_context_server.rs
use anyhow::{bail, Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use notify::event::AccessKind;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::channel,
};

fn main() -> Result<()> {
    // Log startup for debugging
    eprintln!("🔒 sops-context-server starting...");

    // Locate `sops` in PATH
    let sops_path = match which::which("sops") {
        Ok(path) => {
            eprintln!("✅ Found sops at: {}", path.display());
            path
        },
        Err(e) => {
            eprintln!("❌ Could not find sops binary: {}", e);
            bail!("sops binary not found in PATH");
        }
    };

    // Watch entire workspace
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())
        .context("failed to initialize file watcher")?;

    eprintln!("🔍 Watching current directory for changes");
    watcher.watch(Path::new("."), RecursiveMode::Recursive)
        .context("failed to watch workspace")?;

    eprintln!("🔒 sops-context-server running... watching for encrypted files");

    for res in rx {
        match res {
            Ok(Event { kind, paths, .. }) => {
                eprintln!("📝 Event: {:?} on paths: {:?}", kind, paths);

                // Better event handling based on event type
                match kind {
                    EventKind::Access(access_kind) => {
                        match access_kind {
                            AccessKind::Open(_) => {
                                // File was opened
                                for path in paths {
                                    if is_sops_file(&path) {
                                        eprintln!("📂 File opened: {}", path.display());
                                        if let Err(e) = decrypt(&sops_path, &path) {
                                            eprintln!("❌ Decrypt error on open: {}", e);
                                        }
                                    }
                                }
                            },
                            AccessKind::Close(_) => {
                                // File was closed
                                for path in paths {
                                    if is_sops_file(&path) {
                                        eprintln!("🚪 File closed: {}", path.display());
                                        if let Err(e) = encrypt(&sops_path, &path) {
                                            eprintln!("❌ Encrypt error on close: {}", e);
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    EventKind::Modify(_) => {
                        // File was modified/saved
                        for path in paths {
                            if is_sops_file(&path) {
                                eprintln!("💾 File modified: {}", path.display());
                                // When a file is modified and it's a SOPS file,
                                // we should first ensure it's decrypted for editing
                                if let Err(e) = ensure_decrypted(&sops_path, &path) {
                                    eprintln!("❌ Ensure decrypted error: {}", e);
                                }
                            }
                        }
                    },
                    EventKind::Create(_) => {
                        // New file created
                        for path in paths {
                            if is_sops_file(&path) {
                                eprintln!("🆕 New SOPS file created: {}", path.display());
                                if let Err(e) = check_and_process_file(&sops_path, &path) {
                                    eprintln!("❌ Processing error on create: {}", e);
                                }
                            }
                        }
                    },
                    _ => {} // Ignore other events like Remove, Rename, etc.
                }
            },
            Err(e) => eprintln!("⚠️ Watch error: {:?}", e),
        }
    }

    Ok(())
}

fn check_and_process_file(sops: &Path, path: &PathBuf) -> Result<()> {
    // Check if the file appears to be encrypted
    let content = fs::read_to_string(path)?;
    if content.contains("ENC[") || content.contains("sops:") {
        eprintln!("🔍 File appears to be encrypted, decrypting: {}", path.display());
        decrypt(sops, path)?;
    } else {
        eprintln!("📄 File doesn't appear encrypted, monitoring: {}", path.display());
    }
    Ok(())
}

fn ensure_decrypted(sops: &Path, path: &PathBuf) -> Result<()> {
    // Read the content to check if it's already decrypted
    let content = fs::read_to_string(path)?;
    if content.contains("ENC[") {
        eprintln!("🔓 File needs decryption: {}", path.display());
        decrypt(sops, path)?;
    } else {
        eprintln!("✅ File is already decrypted: {}", path.display());
    }
    Ok(())
}

fn is_sops_file(path: &PathBuf) -> bool {
    // Check if the file exists and is a file
    if !path.is_file() {
        return false;
    }

    // Get the file name as a string
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // Check for common SOPS filename patterns
    if file_name.ends_with(".sops.yaml") ||
       file_name.ends_with(".sops.json") ||
       file_name.ends_with(".enc.yaml") ||
       file_name.ends_with(".enc.json") ||
       file_name.ends_with(".sops") {
        eprintln!("📄 Found SOPS file by extension: {}", path.display());
        return true;
    }

    // Check file content (as a fallback)
    if let Ok(content) = fs::read_to_string(path) {
        if (content.contains("sops:") && content.contains("ENC[")) ||
           content.contains("encrypted_suffix") {
            eprintln!("📄 Found SOPS file by content: {}", path.display());
            return true;
        }
    }

    false
}

fn decrypt(sops: &Path, path: &PathBuf) -> Result<()> {
    eprintln!("🔑 Running: {} -d {}", sops.display(), path.display());

    let output = Command::new(sops)
        .arg("-d")
        .arg(path)
        .output()
        .context(format!("running `{} -d {}` failed", sops.display(), path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Decrypt error: {}", stderr);
        bail!("sops decrypt error: {}", stderr);
    }

    fs::write(path, &output.stdout)
        .context("writing decrypted content failed")?;

    eprintln!("✅ Decrypted {}", path.display());
    Ok(())
}

fn encrypt(sops: &Path, path: &PathBuf) -> Result<()> {
    eprintln!("🔒 Running: {} -e -i {}", sops.display(), path.display());

    // First read the current content
    let content = fs::read_to_string(path)?;

    // Only encrypt if it's not already encrypted
    if content.contains("ENC[") {
        eprintln!("⏭️ File already encrypted, skipping: {}", path.display());
        return Ok(());
    }

    let output = Command::new(sops)
        .arg("-e")
        .arg("-i")  // Use in-place editing
        .arg(path)
        .output()
        .context(format!("running `{} -e -i {}` failed", sops.display(), path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Encrypt error: {}", stderr);
        bail!("sops encrypt error: {}", stderr);
    }

    eprintln!("✅ Re-encrypted {}", path.display());
    Ok(())
}
