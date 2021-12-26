use std::path::Path;
use std::{env, fs};
use supershare::{check_and_remove_old_files, SuperShare};
use log::info;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    simple_log::quick_log_level("info", None).unwrap();

    let path = std::env::args()
        .nth(1)
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|| {
            env::current_dir()
                .expect("Failed to get current directory")
                .join("supershare")
        });

    fs::create_dir_all(&path).expect("Failed to created supershare directory");

    info!("Starting SuperShare, files will be stored in: {:?}", path);
    futures::join!(
        SuperShare::new(&path).serve(),
        check_and_remove_old_files(&path)
    );
}
