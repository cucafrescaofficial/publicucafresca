use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(src_path, dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}

fn main() {
    let resources_dir = PathBuf::from("resources");

    let profile = env::var("PROFILE").unwrap();
    
    // Diretório do target (onde os executáveis são gerados)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut target_dir = Path::new(&manifest_dir).join("target");
    
    // Check if a custom target is being used
    if let Ok(target_triple) = env::var("TARGET") {
        target_dir = target_dir.join(&target_triple);
    }
    
    // Add the profile (debug/release)
    target_dir = target_dir.join(&profile);

    // Caminho de destino para a pasta resources
    let dest_dir = target_dir.join("resources");

    if dest_dir.exists() {
        let _ = fs::remove_dir_all(&dest_dir);
    }

    copy_dir_all(&resources_dir, &dest_dir).unwrap();

    // Informar ao Cargo para recompilar se os arquivos na pasta resources forem alterados
    println!("cargo:rerun-if-changed=resources");
}
