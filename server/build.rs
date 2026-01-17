use std::path::Path;
use std::process::Command;

fn main() {
    // Check if frontend/dist exists
    let frontend_dist = Path::new("../frontend/dist");
    let frontend_index = frontend_dist.join("index.html");

    // Check if we should force rebuild (via environment variable)
    let force_rebuild = std::env::var("PORTA_FORCE_REBUILD_FRONTEND")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    // Check if key frontend files are newer than dist
    let package_json = Path::new("../frontend/package.json");
    let vite_config = Path::new("../frontend/vite.config.ts");
    let src_dir = Path::new("../frontend/src");

    let source_newer = (package_json.exists() && is_newer(package_json, &frontend_index))
        || (vite_config.exists() && is_newer(vite_config, &frontend_index))
        || (src_dir.exists() && is_dir_newer(src_dir, &frontend_index));

    // Rebuild frontend if dist doesn't exist, index.html is missing, source is newer, or forced
    let needs_rebuild =
        force_rebuild || !frontend_dist.exists() || !frontend_index.exists() || source_newer;

    if needs_rebuild {
        if force_rebuild {
            println!("cargo:warning=Force rebuilding frontend (PORTA_FORCE_REBUILD_FRONTEND=1)...");
        } else if source_newer {
            println!(
                "cargo:warning=Frontend source files are newer than dist, rebuilding frontend..."
            );
        } else {
            println!("cargo:warning=Frontend dist not found, building frontend...");
        }
        build_frontend();
    } else {
        println!("cargo:warning=Frontend dist is up to date");
    }

    // Re-run build script if frontend source changes
    // This ensures cargo rebuilds when frontend files change
    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=../frontend/tsconfig.json");
    // Also watch dist to trigger rebuild if it's deleted
    println!("cargo:rerun-if-changed=../frontend/dist");
}

fn is_newer(path1: &Path, path2: &Path) -> bool {
    if !path1.exists() || !path2.exists() {
        return true;
    }

    let mtime1 = path1
        .metadata()
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let mtime2 = path2
        .metadata()
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    mtime1 > mtime2
}

fn is_dir_newer(dir: &Path, file: &Path) -> bool {
    if !dir.exists() || !dir.is_dir() {
        return false;
    }

    // Check if any file in src directory is newer than dist/index.html
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if is_dir_newer(&path, file) {
                    return true;
                }
            } else if is_newer(&path, file) {
                return true;
            }
        }
    }

    false
}

fn build_frontend() {
    let output = Command::new("npm")
        .args(["run", "build"])
        .current_dir("../frontend")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Frontend built successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("cargo:warning=Frontend build failed:");
            if !stderr.is_empty() {
                eprintln!("cargo:warning=STDERR: {}", stderr);
            }
            if !stdout.is_empty() {
                eprintln!("cargo:warning=STDOUT: {}", stdout);
            }
            panic!("Frontend build failed");
        }
        Err(e) => {
            eprintln!("cargo:warning=Failed to run npm build: {}", e);
            panic!("Failed to build frontend: {}", e);
        }
    }
}
