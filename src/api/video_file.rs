use super::abstractions::XYXYc;
use super::inference::detect_bbox_from_imgbuf;
use super::render::draw_bbox_from_imgbuf;
use super::rest::detect_bbox_from_buf_remotely;
use super::utils::{image_buffer_to_jpg_buffer, image_buffer_to_ndarray, ndarray_to_image_buffer};
use ndarray::{ArrayBase, Dim, OwnedRepr};
use std::collections::HashMap;
use std::error::Error;
use std::{iter::Iterator, path::Path};
use video_rs::encode::Settings;
use video_rs::{Decoder, DecoderBuilder, Encoder, Time, WriterBuilder};

// pub struct VideoSteamWrapper {
//     videostream: VideoStream,
// }

// unsafe impl Sync for VideoSteamWrapper {}

// impl VideoSteamWrapper {
//     pub fn new(path_or_url: &str) -> Self {
//         let videostream = VideoStream::new(path_or_url);
//         Self { videostream }
//     }

//     pub fn run_exp(&mut self) -> (Vec<u8>, Vec<XYXYc>) {
//         self.videostream.run_exp()
//     }

//     pub fn run_remotely_exp(&mut self, url: &str) -> (Vec<u8>, Vec<XYXYc>) {
//         self.videostream.run_remotely_exp(url)
//     }

//     pub fn ignore_frame(&mut self) {
//         self.videostream.next();
//     }

//     pub fn get_jpg_frame(&mut self) -> Vec<u8> {
//         let img = self.videostream.next().unwrap();
//         let jpg_buffer = image_buffer_to_jpg_buffer(img);
//         return jpg_buffer;
//     }

//     pub fn measure_fps(&mut self, iterations: u32) -> u32 {
//         self.videostream.measure_fps(iterations)
//     }

//     pub fn measure_inference(&mut self, iterations: u32) -> u32 {
//         self.videostream.measure_inference(iterations)
//     }

//     pub fn measure_remote_inference(&mut self, iterations: u32, url: &str) -> u32 {
//         self.videostream.measure_remote_inference(iterations,url)
//     }
// }

pub struct VideofileProcessor {
    decoder: Decoder,
    encoder: Encoder,
}

pub fn get_output_path(file_path: &str) -> String {
    if let Some(pos) = file_path.rfind('\\') {
        let (directory, file_name) = file_path.split_at(pos + 1);
        let new_file_name = format!("predict_{}", file_name);
        format!("{}{}", directory, new_file_name)
    } else {
        format!("predict_{}", file_path)
    }
}

impl VideofileProcessor {
    #[flutter_rust_bridge::frb(sync)]
    pub fn new(file_path: &str) -> Self {
        video_rs::init().unwrap();
        let decoder = DecoderBuilder::new(Path::new(file_path)).build().unwrap();

        let (w, h) = decoder.size();

        let mut options = HashMap::new();
        options.insert(
            "movflags".to_string(),
            "frag_keyframe+empty_moov".to_string(),
        );
        let output_path = get_output_path(&file_path);

        let _writer = WriterBuilder::new(Path::new(&output_path))
            .with_options(&options.into())
            .build()
            .unwrap();

        let settings = Settings::preset_h264_yuv420p(w as _, h as _, false);
        let encoder = Encoder::new(Path::new(&output_path), settings).unwrap();

        Self { decoder, encoder }
    }

    pub fn get_n_frames(&self) -> u64 {
        self.decoder.frames().unwrap()
    }

    // If the annotation is provided, it will just use that instead of computing it.
    fn process_frame<F>(
        &mut self,
        prediction_fn: F,
        vec: Option<Vec<XYXYc>>,
    ) -> Result<(Vec<u8>, Vec<XYXYc>), Box<dyn Error>>
    where
        F: Fn(&image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Vec<XYXYc>,
    {
        match self.next() {
            Some((time, frame)) => {
                let mut img = ndarray_to_image_buffer(&frame);
                let predictions;
                if vec.is_some() {
                    predictions = vec.unwrap();
                } else {
                    predictions = prediction_fn(&img);
                }
                draw_bbox_from_imgbuf(&mut img, &predictions);
                let final_frame = image_buffer_to_ndarray(&img);
                self.encoder.encode(&final_frame, time).unwrap(); // You may want to handle this unwrap as well
                let jpg_buffer = image_buffer_to_jpg_buffer(img);
                Ok((jpg_buffer, predictions))
            }
            None => Err("Failed to retrieve the next frame.".into()), // Handle None case by returning a descriptive error
        }
    }

    fn run(&mut self, vec: Option<Vec<XYXYc>>) -> Result<(Vec<u8>, Vec<XYXYc>), Box<dyn Error>> {
        self.process_frame(|img| detect_bbox_from_imgbuf(img), vec)
    }

    fn run_remotely(
        &mut self,
        url: &str,
        vec: Option<Vec<XYXYc>>,
    ) -> Result<(Vec<u8>, Vec<XYXYc>), Box<dyn Error>> {
        self.process_frame(
            |img| detect_bbox_from_buf_remotely(url.to_string(), img.to_vec()),
            vec,
        )
    }

    pub fn run_exp(&mut self, vec: Option<Vec<XYXYc>>) -> (Vec<u8>, Vec<XYXYc>) {
        self.run(vec).unwrap()
    }

    pub fn run_remotely_exp(
        &mut self,
        url: &str,
        vec: Option<Vec<XYXYc>>,
    ) -> (Vec<u8>, Vec<XYXYc>) {
        self.run_remotely(url, vec).unwrap()
    }
}

impl Iterator for VideofileProcessor {
    type Item = (Time, ArrayBase<OwnedRepr<u8>, Dim<[usize; 3]>>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode_iter().next() {
            Some(Ok((time, frame))) => Some((time, frame)),
            _ => None,
        }
    }
}

// Given a video file_path
// We run inference for each frame then create a new videofile displayingthe predictions
#[flutter_rust_bridge::frb(dart_async)]
pub fn predict_videofile(file_path: &str, n: usize) {
    let mut frame_processor = VideofileProcessor::new(file_path);
    let mut prev_bbox = None;
    let mut frame_count = 0;

    while let Ok((_jpeg, bbox)) = frame_processor.run(if frame_count % n == 0 {
        None
    } else {
        prev_bbox.clone()
    }) {
        if frame_count % n == 0 {
            prev_bbox = Some(bbox);
        }
        frame_count += 1;
    }
}

// Given a video file_path
// We run inference for each frame then create a new videofile displayingthe predictions
#[flutter_rust_bridge::frb(dart_async)]
pub fn predict_videofile_remotely(file_path: &str, url: &str, n: usize) {
    let mut frame_processor = VideofileProcessor::new(file_path);
    let mut prev_bbox = None;
    let mut frame_count = 0;

    while let Ok((_jpeg, bbox)) = frame_processor.run_remotely(
        url,
        if frame_count % n == 0 {
            None
        } else {
            prev_bbox.clone()
        },
    ) {
        if frame_count % n == 0 {
            prev_bbox = Some(bbox);
        }
        frame_count += 1;
    }
}
