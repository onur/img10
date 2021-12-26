use supershare::{SuperShare, check_and_remove_old_files};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let path = "/tmp/supershare";

    futures::join!(
        SuperShare::new(&path).serve(),
        check_and_remove_old_files(&path)
    );
}
