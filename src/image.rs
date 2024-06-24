use std::{collections::HashMap, f32::consts::E, fs::{self, File}, io::{self, Write}, path::PathBuf, sync::{mpsc::{Receiver, Sender}, Arc, Mutex}};

use image::{io::Reader as ImageReader, DynamicImage};

use directories::ProjectDirs;
use egui::{ColorImage, TextureHandle, TextureId, TextureOptions};


pub enum LoadableImage {
    /// Completely unplanned, doesn't exist
    Unloaded,
    /// In progress
    Loading,
    /// Here ya go
    Loaded(TextureId)
}

enum LoaderRequest {
    Shutdown,
    GetImg(String)
}

pub struct ResDbImageCache {
    cache_path: Option<PathBuf>,
    db: Arc<Mutex<HashMap<String, Option<TextureHandle>>>>,

    tx: Sender<LoaderRequest>,

    ctx: egui::Context,
}

impl ResDbImageCache {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx0, rx1) = std::sync::mpsc::channel();

        let proj_dirs = ProjectDirs::from("com", "hedassbtw",  "ResBox");
        if proj_dirs.is_none() { println!("Could not create image cache folder"); return Self { cache_path: None, db: Arc::new(Mutex::new(HashMap::new())), tx: tx0, ctx}; }
        let proj_dirs = proj_dirs.unwrap();
        
        let dir = proj_dirs.data_local_dir().join("image_cache");
        if !dir.exists() { fs::create_dir_all(&dir).unwrap(); }
        
        let cache = dir.clone();
        
        let map: Arc<Mutex<HashMap<String, Option<TextureHandle>>>> = Arc::new(Mutex::new(HashMap::new()));
        let map0 = map.clone();
        let ctx0 = ctx.clone();
        
        tokio::task::spawn(async move {
            let _result = ResDbImageCache::run(rx1, map0, cache, ctx0).await;
        });

        Self { cache_path: Some(dir), db: map, tx: tx0, ctx}
    }
    
    async fn run(
        rx1: Receiver<LoaderRequest>,
        map: Arc<Mutex<HashMap<String, Option<TextureHandle>>>>,
        cache: PathBuf,
        ctx: egui::Context
    ) -> anyhow::Result<()>  {
        println!("image cache dir: {}", cache.to_str().unwrap());
        let client = reqwest::Client::builder().user_agent("some fuckass rust app that looks like the 2015 xbox one guide").build();
        if let Err(err) = client { return Err(anyhow::Error::msg(format!("{:?}", err))); }
        let client = client.unwrap();
        'outer: loop {
            while let Ok(req) = rx1.try_recv() {
                let req = match req {
                    LoaderRequest::Shutdown => break 'outer Ok(()),
                    LoaderRequest::GetImg(resdb_path) => resdb_path,
                };
                
                if req.is_empty() || !req.contains(".") { println!("empty or dot: {}", req); continue; }
                let split_idx = if let Some(pos) = req.find("://") { pos } else { println!("no beginner: {}", req); continue; };
                let (prefix, path) = req.split_at(split_idx+(if req.find(":///").is_some() {4} else {3}));
                
                // idk why but i'm gonna support HTTP urls too!
                let web_path = if prefix.eq("resdb:///") {
                    let (important, _webp) = path.split_at(path.find(".").unwrap());
                    format!("https://assets.resonite.com/{}", important)
                } else if prefix.starts_with("http") {
                    format!("{}{}", prefix, path)
                } else {
                    println!("doesn't match spec: {} {}", prefix, path);
                    continue;
                };
                
                let mut file_path = cache.clone();
                file_path.push(path.replace("/", ""));

                if !file_path.exists() {
                    let dl = client.get(&web_path).send().await;

                    if let Err(err) = dl {
                        println!("Failed to download {}! Reason: {:?}", &web_path, &err);
                        continue;
                    } let dl = dl.unwrap();

                    let body = dl.bytes().await?;
                    let file = File::create(&file_path);
                    if file.is_err() {
                        println!("Failed to create {:?}! Reason: {:?}", &file_path, &file);
                        continue;
                    } let mut file = file.unwrap();

                    if let Err(err) = file.write_all(&body) {
                        println!("Failed to copy file! Reason: {:?}", err);
                        continue;
                    }
                }

                let file_read = Self::load_from_fs(ctx.clone(), &file_path);

                if let Ok(fil) = file_read {
                    let mut map = map.lock().unwrap();
                    map.insert(req.clone(), Some(fil));
                } else if let Err(err) = file_read {
                    println!("Failed to read image! {:?}", err);
                }
            }
        }
    }

    pub fn load_from_fs(ctx: egui::Context, path: &PathBuf) -> anyhow::Result<TextureHandle> {

        let identifier = path.file_name().unwrap().to_str().unwrap();

        println!("Loading image {:?}", path);
        let img = ImageReader::open(path);
        if img.is_err() {
            return Err(anyhow::Error::msg(format!("Failed to open \"{}\"!", path.to_string_lossy())));
        }

        let img_decoded = img?.decode();
        if img_decoded.is_err() {
            return Err(anyhow::Error::msg("Failed to decode image"));
        }

        let img_decoded = img_decoded?;

        match img_decoded.color().channel_count() {
            2 => {
                let img_a = DynamicImage::ImageRgba8(img_decoded.into_rgba8());
                let ci = ColorImage::from_rgba_unmultiplied(
                    [img_a.width() as usize, img_a.height() as usize],
                    img_a.as_bytes(),
                );

                return Ok(ctx.load_texture(identifier, ci, TextureOptions::NEAREST));
                
            }
            3 => {
                let ci = ColorImage::from_rgb(
                    [img_decoded.width() as usize, img_decoded.height() as usize],
                    img_decoded.as_bytes(),
                );
                
                return Ok(ctx.load_texture(identifier, ci, TextureOptions::NEAREST));
            }
            4 => {
                let ci = ColorImage::from_rgba_unmultiplied(
                    [img_decoded.width() as usize, img_decoded.height() as usize],
                    img_decoded.as_bytes(),
                );

                return Ok(ctx.load_texture(identifier, ci, TextureOptions::NEAREST));
            }
            _ => return Err(anyhow::Error::msg("unsupported amount of channels")),
        }
    }

    /// tells the thread to \*lightning\*
    pub fn shutdown(&mut self) {
        self.tx.send(LoaderRequest::Shutdown).unwrap();
    }

    /// Accepts a `resdb://` string and gets an egui-drawable image (or lack thereof) from it
    pub fn get_image(&mut self, id: &String) -> LoadableImage {
        let mut db = self.db.lock().unwrap();
        if let Some(img) = db.get(id) {
            if let Some(id) = img {
                return LoadableImage::Loaded(id.id());
            } else {
                return LoadableImage::Loading;
            }
        } else {
            // i don't want another enum so i just use existing as unloaded, and the option as loading/loaded
            // race conditions arise from setting None on the thread (loader thread is busy loading, and doesn't set None itself)
            // so we do it here
            db.insert(id.clone(), None);
            if let Err(err) = self.tx.send(LoaderRequest::GetImg(id.to_string())) {
                println!("Send error! {:?}", err);
            }
        }
        

        LoadableImage::Unloaded
    }
}