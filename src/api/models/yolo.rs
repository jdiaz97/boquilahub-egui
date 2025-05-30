use super::*;
use crate::api::{
    abstractions::{BoundingBoxTrait, XYXY},
    bq::import_bq,
};
use image::{
    imageops::{resize, FilterType},
    ImageBuffer, Rgb,
};
use ndarray::{s, Array, Axis, Ix4, IxDyn};
use ort::{
    inputs,
    session::{builder::GraphOptimizationLevel, Session},
};

pub struct Yolo {
    pub name: String,
    pub description: String,
    pub version: f32,
    pub classes: Vec<String>,
    pub input_width: u32,
    pub input_height: u32,
    pub confidence_threshold: f32,
    pub nms_threshold: f32,
    pub num_classes: u32,
    pub num_masks: u32,
    pub task: Task,
    pub session: Session,
}

impl Yolo {
    pub fn new(
        name: String,
        description: String,
        version: f32,
        classes: Vec<String>,
        input_width: u32,
        input_height: u32,
        confidence_threshold: f32,
        nms_threshold: f32,
        num_classes: u32,
        num_masks: u32,
        task: Task,
        session: Session,
    ) -> Self {
        Self {
            name,
            description,
            version,
            classes,
            input_width,
            input_height,
            confidence_threshold,
            nms_threshold,
            num_classes,
            num_masks,
            task,
            session,
        }
    }

    pub fn default() -> Self {
        let (_, data) = import_bq("models/boquilanet-gen.bq").unwrap();
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_memory(&data)
            .unwrap();

        Yolo::new(
            "boquilanet-gen".to_string(),
            "Generic animal detection".to_string(),
            0.1,
            vec!["animal".to_string()],
            1024,
            1024,
            0.45,
            0.5,
            1,
            0,
            Task::Detect,
            session,
        )
    }

    fn prepare_input_from_buf(&self, buf: &[u8]) -> (Array<f32, Ix4>, u32, u32) {
        let img = image::load_from_memory(buf).unwrap().into_rgb8();
        self.prepare_input_from_imgbuf(&img)
    }

    fn prepare_input_from_imgbuf(
        &self,
        img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> (Array<f32, Ix4>, u32, u32) {
        let (img_width, img_height) = (img.width(), img.height());

        let resized = resize(
            img,
            self.input_width,
            self.input_height,
            FilterType::Nearest,
        );

        let mut input = Array::zeros((1, 3, self.input_height as usize, self.input_width as usize));

        for (x, y, pixel) in resized.enumerate_pixels() {
            let x_u = x as usize;
            let y_u = y as usize;
            input[[0, 2, y_u, x_u]] = (pixel[2] as f32) / 255.0;
            input[[0, 1, y_u, x_u]] = (pixel[1] as f32) / 255.0;
            input[[0, 0, y_u, x_u]] = (pixel[0] as f32) / 255.0;
        }

        (input, img_width, img_height)
    }

    fn run_detect(&self, input: &Array<f32, Ix4>) -> Array<f32, IxDyn> {
        let outputs = self
            .session
            .run(inputs!["images" => input.view()].unwrap())
            .unwrap();

        let predictions = outputs["output0"]
            .try_extract_tensor::<f32>()
            .unwrap()
            .t()
            .into_owned();
        return predictions;
    }

    fn process_detect_output(
        &self,
        output: &Array<f32, IxDyn>,
        img_width: u32,
        img_height: u32,
    ) -> Vec<XYXYc> {
        let mut boxes = Vec::new();
        let output = output.slice(s![.., .., 0]);
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<f32> = row.iter().map(|x| *x).collect();
            let (class_id, prob) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();
            if prob < self.confidence_threshold {
                continue;
            }
            let label = class_id as u16;
            // XYWHn::new(row[0],row[1],row[0],row[3],prob,label);
            let xc = row[0] / self.input_width as f32 * (img_width as f32);
            let yc = row[1] / self.input_height as f32 * (img_height as f32);
            let w = row[2] / self.input_width as f32 * (img_width as f32);
            let h = row[3] / self.input_height as f32 * (img_height as f32);
            let x1 = xc - w / 2.0;
            let x2 = xc + w / 2.0;
            let y1 = yc - h / 2.0;
            let y2 = yc + h / 2.0;
            let temp = XYXY::new(x1, y1, x2, y2, prob, label);
            boxes.push(temp);
        }

        let indices = nms_indices(&boxes, self.nms_threshold);
        let result: Vec<XYXY> = indices.iter().map(|&idx| boxes[idx].clone()).collect();
        return self.t(&result);
    }

    fn t(&self, boxes: &Vec<XYXY>) -> Vec<XYXYc> {
        boxes
            .into_iter()
            .map(|xyxy| {
                let label = &self.classes[xyxy.get_class_id() as usize];
                xyxy.to_xyxyc(None, None, label.to_string())
            })
            .collect()
    }

    pub fn run(&self, img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> AIOutputs {
        let (input, img_width, img_height) = self.prepare_input_from_imgbuf(img);
        match self.task {
            Task::Detect => {
                let output = self.run_detect(&input);
                let boxes = self.process_detect_output(&output, img_width, img_height);
                return AIOutputs::ObjectDetection(boxes);
            }
            Task::Classify => {
                todo!();
            }
            Task::Segment => {
                todo!();
            }
        }
    }
}

fn nms_indices<T: BoundingBoxTrait>(boxes: &[T], iou_threshold: f32) -> Vec<usize> {
    // Create indices and sort them by probability (descending)
    let mut indices: Vec<usize> = (0..boxes.len()).collect();
    indices.sort_by(|&a, &b| {
        boxes[b]
            .get_prob()
            .partial_cmp(&boxes[a].get_prob())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut keep = Vec::new();

    while !indices.is_empty() {
        // Keep the highest scoring box
        let current_idx = indices[0];
        keep.push(current_idx);

        // Filter remaining indices
        indices = indices
            .into_iter()
            .skip(1)
            .filter(|&idx| {
                boxes[idx].get_class_id() != boxes[current_idx].get_class_id()
                    || boxes[idx].iou(&boxes[current_idx]) <= iou_threshold
            })
            .collect();
    }

    keep
}
