//use rand::Rng;
//use warp::{Buf, Filter};
//use cfb8::Cfb8;
//use cfb8::cipher::{NewCipher, AsyncStreamCipher};
//use aes::Aes128;
//use std::io::{Read, Write};
//use std::fs::File;
//
//const ID_SIZE: usize = 16;
//
//fn gen_id() -> String {
//    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
//    let mut rng = rand::thread_rng();
//    (0..ID_SIZE)
//        .map(|_| rng.gen::<usize>() % chars.len())
//        .map(|i| chars.chars().nth(i).unwrap())
//        .collect()
//}
//
//fn upload(filename: String, mut body: impl Buf) -> String {
//    let (key, iv) = (gen_id(), gen_id());
//    let mut cipher = Cfb8::<Aes128>::new_from_slices(key.as_bytes(), iv.as_bytes()).unwrap();
//    let mut file = File::create(format!("/tmp/{}", key)).unwrap();
//
//    while body.has_remaining() {
//        let mut chunk = body.chunk().to_vec();
//        cipher.encrypt(&mut chunk);
//        file.write_all(&chunk).unwrap();
//        let cnt = body.chunk().len();
//        body.advance(cnt);
//    }
//
//    println!("{} {}", key, iv);
//    "OK".to_string()
//}
//
//fn download(key: String, iv: String) -> String {
//    let mut cipher = Cfb8::<Aes128>::new_from_slices(key.as_bytes(), iv.as_bytes()).unwrap();
//    let mut file = File::open(format!("/tmp/{}", key)).unwrap();
//
//    let mut output_file = File::create("/tmp/output").unwrap();
//
//    loop {
//        let mut buffer = [0; 128];
//        let n = file.read(&mut buffer).unwrap();
//        cipher.decrypt(&mut buffer[0..n]);
//        output_file.write_all(&buffer[0..n]).unwrap();
//        if n < buffer.len() {
//            break;
//        }
//    }
//
//    "OK".to_string()
//}
//
//#[tokio::main(flavor = "current_thread")]
//async fn main() {
//    let upload = warp::path!(String).and(warp::put()).and(warp::body::aggregate()).map(upload);
//    let download = warp::path!(String / String).map(download);
//
//    let routes = upload.or(download);
//    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
//}

use supershare::SuperShare;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let supershare = SuperShare::new("/tmp/supershare");
    supershare.serve().await
}
