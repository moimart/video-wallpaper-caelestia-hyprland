use gtk::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::services::video_scanner::VideoEntry;

const TILE_SIZE: i32 = 240;

pub struct VideoTile {
    pub widget: gtk::Frame,
    pub video_path: PathBuf,
    picture: gtk::Picture,
    thumbnail_path: PathBuf,
    media_stream: Rc<RefCell<Option<gtk::MediaFile>>>,
    graveyard: Rc<RefCell<Vec<gtk::MediaFile>>>,
}

impl VideoTile {
    pub fn new(video: &VideoEntry, thumbnail_path: &Path, is_selected: bool) -> Self {
        let widget = gtk::Frame::builder()
            .css_classes(if is_selected {
                vec!["video-tile", "selected"]
            } else {
                vec!["video-tile"]
            })
            .overflow(gtk::Overflow::Hidden)
            .build();
        widget.set_size_request(TILE_SIZE, TILE_SIZE);

        let picture = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .width_request(TILE_SIZE)
            .height_request(TILE_SIZE)
            .build();

        if thumbnail_path.exists() {
            picture.set_filename(Some(thumbnail_path));
        }

        widget.set_child(Some(&picture));

        Self {
            widget,
            video_path: video.path.clone(),
            picture,
            thumbnail_path: thumbnail_path.to_path_buf(),
            media_stream: Rc::new(RefCell::new(None)),
            graveyard: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn start_preview(&self) {
        if self.media_stream.borrow().is_some() {
            return;
        }
        let mf = gtk::MediaFile::for_filename(&self.video_path);
        mf.set_muted(true);
        mf.set_loop(true);

        let pic = self.picture.clone();
        mf.connect_notify_local(Some("prepared"), move |mf, _| {
            if mf.is_prepared() {
                pic.set_paintable(Some(mf));
            }
        });

        mf.play();
        *self.media_stream.borrow_mut() = Some(mf);
    }

    pub fn stop_preview(&self) {
        if let Some(mf) = self.media_stream.borrow_mut().take() {
            mf.pause();
            self.graveyard.borrow_mut().push(mf);
        }
        self.picture.set_paintable(gdk::Paintable::NONE);
        if self.thumbnail_path.exists() {
            self.picture.set_filename(Some(&self.thumbnail_path));
        }
    }

    /// Forget all GStreamer MediaFile objects so they are never freed by Rust.
    /// Called before GTK window teardown to prevent the NVIDIA GL driver from
    /// crashing when `g_object_unref` races with GStreamer's background GL thread.
    /// The OS reclaims everything when the process exits moments later.
    pub fn release_media(&self) {
        self.picture.set_paintable(gdk::Paintable::NONE);
        if let Some(mf) = self.media_stream.borrow_mut().take() {
            mf.pause();
            std::mem::forget(mf);
        }
        for mf in self.graveyard.borrow_mut().drain(..) {
            std::mem::forget(mf);
        }
    }
}
