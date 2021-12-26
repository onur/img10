use supershare::SuperShare;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    let supershare = SuperShare::new("/tmp/supershare");
    supershare.serve().await
}
