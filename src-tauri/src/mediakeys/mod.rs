use log::{info, warn};
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::{Id, Retained};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::{CGSize, NSDictionary, NSNumber, NSObject, NSString, NSUInteger};

use crate::metadata::{Artwork, Song};
extern "C" {
    static MPMediaItemPropertyTitle: *const NSString;
    static MPMediaItemPropertyArtist: *const NSString;
    static MPMediaItemPropertyAlbumTitle: *const NSString;
    static MPMediaItemPropertyPlaybackDuration: *const NSString;
    static MPMediaItemPropertyArtwork: *const NSString;
}
/// Sets the now playing information using an immutable `NSDictionary`.
pub fn set_now_playing_info(song: &Song) {
    info!("Setting now playing info");
    // Create NSStrings for the title, artist, and album.
    let title_nsstring = NSString::from_str(&song.title);
    let artist_nsstring = NSString::from_str(&song.artist);
    let album_nsstring = NSString::from_str(&song.album);
    let duration_nsnumber = NSNumber::new_f64(song.file_info.duration.unwrap_or(0.0f64));

    unsafe {
        // // Use the extern NSString constants as keys.
        let title_key = &*MPMediaItemPropertyTitle;
        let artist_key = &*MPMediaItemPropertyArtist;
        let album_key = &*MPMediaItemPropertyAlbumTitle;
        let duration_key = &*MPMediaItemPropertyPlaybackDuration;
        let artwork_key = &*MPMediaItemPropertyArtwork;

        let mut keys = [title_key, artist_key, album_key, duration_key].to_vec();

        let mut objects = [
            Id::cast(title_nsstring),
            Id::cast(artist_nsstring),
            Id::cast(album_nsstring),
            Id::cast(duration_nsnumber),
        ]
        .to_vec();

        if let Some(art) = Retained::from_raw(set_artwork(&song.artwork)) {
            keys.push(artwork_key);
            objects.push(art);
        }

        // Create an NSDictionary with these key-value pairs.
        let now_playing_info: Retained<NSDictionary<NSString, NSObject>> =
            NSDictionary::from_vec(&keys, objects);

        // Get the MPNowPlayingInfoCenter class and its defaultCenter singleton.
        let info_center: *mut NSObject = msg_send![class!(MPNowPlayingInfoCenter), defaultCenter];
        info!("MPNowPlayingInfoCenter: {:?}", info_center);
        info!("Setting now playing info: {:?}", now_playing_info);
        // Clear the nowPlayingInfo property.
        let nil: *const AnyObject = ptr::null();
        let _: () = msg_send![info_center, setNowPlayingInfo: nil];

        // Set the nowPlayingInfo property using the NSDictionary.
        let _: () = msg_send![info_center, setNowPlayingInfo: &*now_playing_info];
    }
}

pub fn set_playing() {
    unsafe {
        const PLAYING: NSUInteger = 1;
        let info_center: *mut NSObject = msg_send![class!(MPNowPlayingInfoCenter), defaultCenter];
        let _: () = msg_send![info_center, setPlaybackState: PLAYING];
    }
}

pub fn set_paused() {
    unsafe {
        const PAUSED: NSUInteger = 0;
        let info_center: *mut NSObject = msg_send![class!(MPNowPlayingInfoCenter), defaultCenter];
        let _: () = msg_send![info_center, setPlaybackState: PAUSED];
    }
}

pub struct RemoteCommandCenter {
    next_handler: Option<Rc<dyn Fn()>>,
    pause_handler: Option<Rc<dyn Fn()>>,
    play_handler: Option<Rc<dyn Fn()>>,
    previous_handler: Option<Rc<dyn Fn()>>,
    toogle_handler: Option<Rc<dyn Fn()>>,
}

impl RemoteCommandCenter {
    pub fn new() -> Self {
        RemoteCommandCenter {
            next_handler: None,
            pause_handler: None,
            play_handler: None,
            previous_handler: None,
            toogle_handler: None,
        }
    }

    // Set handlers as closures
    pub fn set_handlers<F, G, H, I, J>(
        &mut self,
        play: F,
        pause: G,
        toogle: H,
        previous: I,
        next: J,
    ) where
        F: Fn() + 'static,
        G: Fn() + 'static,
        H: Fn() + 'static,
        I: Fn() + 'static,
        J: Fn() + 'static,
    {
        self.next_handler = Some(Rc::new(next));
        self.pause_handler = Some(Rc::new(pause));
        self.play_handler = Some(Rc::new(play));
        self.previous_handler = Some(Rc::new(previous));
        self.toogle_handler = Some(Rc::new(toogle));
    }

    pub fn setup_remote_command_center(&self) {
        println!("MPRemoteCommandCenter:");
        unsafe {
            let command_center: *mut NSObject =
                msg_send![class!(MPRemoteCommandCenter), sharedCommandCenter];
            println!("MPRemoteCommandCenter: {:?}", command_center);

            // Create the play handler block using the closure
            let play_handler_clone = self.play_handler.clone();
            let play_handler_block = RcBlock::new(move |_command: *mut NSObject| {
                if let Some(handler) = &play_handler_clone {
                    handler(); // Call the play handler closure
                }
                Id::as_ptr(&NSNumber::new_i32(0)) // Return NSNumber indicating success
            });

            // Register the play command with the block.
            let play_command: *mut NSObject = msg_send![command_center, playCommand];
            let _: *mut NSObject =
                msg_send![play_command, addTargetWithHandler: &*play_handler_block];

            // Create the pause handler block using the closure
            let pause_handler_clone = self.pause_handler.clone();
            let pause_handler_block = RcBlock::new(move |_command: *mut NSObject| {
                if let Some(handler) = &pause_handler_clone {
                    handler(); // Call the pause handler closure
                }
                Id::as_ptr(&NSNumber::new_i32(0)) // Return NSNumber indicating success
            });

            // Register the pause command with the block.
            let pause_command: *mut NSObject = msg_send![command_center, pauseCommand];
            let _: *mut NSObject =
                msg_send![pause_command, addTargetWithHandler: &*pause_handler_block];

            // Create the previous handler block using the closure
            let previous_handler_clone = self.previous_handler.clone();
            let previous_handler_block = RcBlock::new(move |_command: *mut NSObject| {
                if let Some(handler) = &previous_handler_clone {
                    handler(); // Call the play handler closure
                }
                Id::as_ptr(&NSNumber::new_i32(0)) // Return NSNumber indicating success
            });

            // Register the previous command with the block.
            let previous_command: *mut NSObject = msg_send![command_center, previousTrackCommand];
            let _: *mut NSObject =
                msg_send![previous_command, addTargetWithHandler: &*previous_handler_block];

            // Create the toogle handler block using the closure
            let toogle_handler_clone = self.toogle_handler.clone();
            let toogle_handler_block = RcBlock::new(move |_command: *mut NSObject| {
                if let Some(handler) = &toogle_handler_clone {
                    handler(); // Call the play handler closure
                }
                Id::as_ptr(&NSNumber::new_i32(0)) // Return NSNumber indicating success
            });

            // Register the toogle command with the block.
            let toogle_command: *mut NSObject = msg_send![command_center, togglePlayPauseCommand];
            let _: *mut NSObject =
                msg_send![toogle_command, addTargetWithHandler: &*toogle_handler_block];

            // Create the next handler block using the closure
            let next_handler_clone = self.next_handler.clone();
            let next_handler_block = RcBlock::new(move |_command: *mut NSObject| {
                if let Some(handler) = &next_handler_clone {
                    handler(); // Call the play handler closure
                }
                Id::as_ptr(&NSNumber::new_i32(0)) // Return NSNumber indicating success
            });

            // Register the next command with the block.
            let next_command: *mut NSObject = msg_send![command_center, nextTrackCommand];
            let _: *mut NSObject =
                msg_send![next_command, addTargetWithHandler: &*next_handler_block];
        }
    }
}

unsafe fn set_artwork(artwork: &Option<Artwork>) -> *mut NSObject {
    // Create an NSImage from either the artwork data or the file path.
    let ns_image_class = class!(NSImage);
    let mut ns_image: *mut NSObject;
    if let Some(art) = artwork {
        if art.data.len() > 0 {
            info!("Now playing artwork: Data length: {}", art.data.len());
            // Create NSData from the raw artwork data.
            let ns_data_class = class!(NSData);
            let artdata_pointer = art.data.as_ptr() as *const c_void;
            let ns_artwork_data: *mut NSObject =
                msg_send![ns_data_class, dataWithBytes:artdata_pointer length:art.data.len()];
            info!("NSData: {:?}", ns_artwork_data);
            // Create NSImage from NSData.
            ns_image = msg_send![ns_image_class, alloc];
            ns_image = msg_send![ns_image, initWithData: ns_artwork_data];
        } else if let Some(file_path) = &art.src {
            info!("Now playing artwork: File path: {}", file_path);
            // Create an NSString for the file path.
            let file_path_nsstring = NSString::from_str(file_path.as_str());
            // Initialize NSImage using the file path.
            ns_image = msg_send![ns_image_class, alloc];
            ns_image = msg_send![ns_image, initByReferencingFile: &*file_path_nsstring];
        } else {
            warn!("No artwork data or file path provided; skipping artwork setup.");
            return ptr::null_mut();
        }
    } else {
        return ptr::null_mut();
    }
    info!("Setting Now Playing Artwork");
    info!("NSImage: {:?}", ns_image);
    // Create an MPMediaItemArtwork instance from the NSImage.
    let media_artwork_class = class!(MPMediaItemArtwork);
    // Alloc
    let media_artwork_class_alloc: *mut NSObject = msg_send![media_artwork_class, alloc];

    // Create the request handler block. This block receives a CGSize and returns the NSImage.
    let handler_block = RcBlock::new(move |size: *const CGSize| -> *const NSObject {
        info!("Size: {:?}", size);
        ns_image
    });
    let _size = CGSize::new(200f64, 200f64);

    let artwork: *mut NSObject = msg_send![media_artwork_class_alloc, initWithBoundsSize: _size requestHandler: &*handler_block];

    return artwork;
}
