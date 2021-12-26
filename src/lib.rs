mod mime_types;

use aes::Aes128;
use cfb8::cipher::{AsyncStreamCipher, NewCipher};
use cfb8::Cfb8;
use futures::stream::Stream;
use futures::task::{Context, Poll};
use log::error;
use rand::Rng;
use std::error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs::{metadata, File};
use tokio::io::AsyncWriteExt;
use warp::http::Response;
use warp::{Buf, Filter};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct SuperShare {
    path: PathBuf,
}

impl SuperShare {
    pub fn new(path: impl AsRef<Path>) -> SuperShare {
        SuperShare {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn gen_id(&self) -> String {
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::thread_rng();
        (0..16)
            .map(|_| rng.gen::<usize>() % chars.len())
            .map(|i| chars.chars().nth(i).unwrap())
            .collect()
    }

    async fn upload(&self, mut body: impl Buf) -> (String, String) {
        let (key, iv) = (self.gen_id(), self.gen_id());
        let mut cipher = Cfb8::<Aes128>::new_from_slices(key.as_bytes(), iv.as_bytes()).unwrap();
        let mut file = File::create(self.path.join(&iv)).await.unwrap();

        while body.has_remaining() {
            let mut chunk = body.chunk().to_vec();
            cipher.encrypt(&mut chunk);
            file.write_all(&chunk).await.unwrap();
            let cnt = body.chunk().len();
            body.advance(cnt);
        }

        (key, iv)
    }

    async fn download(
        &self,
        key: String,
        iv: String,
        filename: String,
    ) -> Result<Response<warp::hyper::Body>> {
        let file = std::fs::File::open(self.path.join(&iv))?;
        let metadata = metadata(self.path.join(&iv)).await?;
        let cipher = Cfb8::<Aes128>::new_from_slices(key.as_bytes(), iv.as_bytes())
            .map_err(|e| format!("Failed to create cipher: {}", e))?;
        let decipher = Decipher { cipher, file };
        let body = warp::hyper::Body::wrap_stream(decipher);

        Ok(Response::builder()
            .header("Content-Length", metadata.len())
            .header("Content-Type", mime_types::from_file_name(&filename))
            .body(body)
            .unwrap())
    }

    pub async fn serve(self) {
        let ss = Arc::new(self);

        fn with_ss(
            ss: Arc<SuperShare>,
        ) -> impl Filter<Extract = (Arc<SuperShare>,), Error = std::convert::Infallible> + Clone
        {
            warp::any().map(move || ss.clone())
        }

        let upload = warp::path!(String)
            .and(warp::put())
            .and(warp::body::aggregate())
            .and(warp::host::optional())
            .and(with_ss(ss.clone()))
            .and_then(|path, body, host, ss: Arc<SuperShare>| async move {
                if !Path::new(&path).is_relative() {
                    return Err(warp::reject::not_found());
                }
                let (key, iv) = ss.upload(body).await;
                if let Some(host) = host {
                    Ok(format!("http://{}/{}/{}/{}\n", host, key, iv, path))
                } else {
                    Ok(format!("{}/{}/{}\n", key, iv, path))
                }
            });

        let download = warp::path!(String / String / String)
            .and(warp::get())
            .and(with_ss(ss.clone()))
            .and_then(|key, iv, filename, ss: Arc<SuperShare>| async move {
                ss.download(key, iv, filename).await.map_err(|e| {
                    error!("Download: {}", e);
                    warp::reject::not_found()
                })
            });

        #[cfg(not(debug_assertions))]
        let index = warp::path::end()
            .and(warp::get())
            .map(|| include_str!("upload.html"));
        #[cfg(debug_assertions)]
        let index = warp::path::end()
            .and(warp::get())
            .and(warp::fs::file("src/upload.html"));

        let routes = upload.or(download).or(index);
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await
    }
}

struct Decipher {
    cipher: Cfb8<Aes128>,
    file: std::fs::File,
}

impl Stream for Decipher {
    type Item = std::result::Result<bytes::Bytes, String>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = [0; 4096];
        let n = self.file.read(&mut buffer).unwrap();
        self.cipher.decrypt(&mut buffer);

        if n > 0 {
            Poll::Ready(Some(Ok(bytes::Bytes::from(buffer[0..n].to_vec()))))
        } else {
            Poll::Ready(None)
        }
    }
}
