use std::path::Path;
use std::{env, fs};
use supershare::{check_and_remove_old_files, SuperShare};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let path = std::env::args()
        .nth(1)
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|| {
            env::current_dir()
                .expect("Failed to get current directory")
                .join("supershare")
        });

    fs::create_dir_all(&path).expect("Failed to created supershare directory");

    futures::join!(
        SuperShare::new(&path).serve(),
        check_and_remove_old_files(&path)
    );
}
