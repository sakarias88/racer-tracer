// TODO : Rename this file

use crate::{
    data_bus::{DataBus, DataReader, DataWriter},
    error::TracerError,
    image::Image,
    tone_map::ToneMap,
    vec3::Color,
};

// There should be more buffers eventually. Will probably need
// z-buffer, pos, normal, object_id and albedo later.
pub struct ImageBufferReader {
    reader: DataReader<ImageBufferEvent>,
    changed: bool,
    image_width: usize,
    rgb: Vec<Color>,
}

impl ImageBufferReader {
    pub fn new(image: &Image, reader: DataReader<ImageBufferEvent>) -> Self {
        Self {
            image_width: image.width,
            rgb: vec![Color::default(); image.width * image.height],
            reader,
            changed: false,
        }
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.reader.get_messages().map(|messages| {
            self.changed = !messages.is_empty();
            messages.into_iter().for_each(|event| match event {
                ImageBufferEvent::BufferUpdate {
                    rgb,
                    r,
                    c,
                    width,
                    height,
                } => {
                    for row in 0..height {
                        for column in 0..width {
                            let buffer_index = row * width + column;
                            let index = (r + row) * self.image_width + c + column;
                            self.rgb[index] = rgb[buffer_index];
                        }
                    }
                }
            })
        })
    }

    pub fn changed(&mut self) -> bool {
        let res = self.changed;
        self.changed = false;
        res
    }

    pub fn rgb(&self) -> &[Color] {
        &self.rgb
    }
}

#[derive(Clone)]
pub enum ImageBufferEvent {
    BufferUpdate {
        rgb: Vec<Color>,
        r: usize,
        c: usize,
        width: usize,
        height: usize,
    },
}

pub struct ImageBuffer {
    bus: DataBus<ImageBufferEvent>,
    image: Image,
}

impl ImageBuffer {
    pub fn new(image: Image) -> Self {
        Self {
            image,
            bus: DataBus::<ImageBufferEvent>::new("ImageBuffer"),
        }
    }

    pub fn get_writer(&self) -> ImageBufferWriter {
        ImageBufferWriter::new(self.bus.get_writer())
    }

    pub fn get_reader(&mut self) -> ImageBufferReader {
        ImageBufferReader::new(&self.image, self.bus.get_reader())
    }

    pub fn get_data_reader(&mut self) -> DataReader<ImageBufferEvent> {
        self.bus.get_reader()
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.bus.update()
    }
}

#[derive(Clone)]
pub struct ImageBufferWriter {
    writer: DataWriter<ImageBufferEvent>,
}

impl ImageBufferWriter {
    pub fn new(writer: DataWriter<ImageBufferEvent>) -> Self {
        Self { writer }
    }

    pub fn write(
        &mut self,
        rgb: Vec<Color>,
        r: usize,
        c: usize,
        width: usize,
        height: usize,
    ) -> Result<(), TracerError> {
        self.writer.write(ImageBufferEvent::BufferUpdate {
            rgb,
            r,
            c,
            width,
            height,
        })
    }
}

// The point of this is to combine all sources to the finished image source
pub struct ScreenBuffer {
    buffer: Vec<Color>,
    out: DataWriter<ImageBufferEvent>,
    reader: DataReader<ImageBufferEvent>,
    bus: DataBus<ImageBufferEvent>,
    image: Image,

    // TODO: Have something more generic that could apply many
    // different pp effects. A list of something.
    // It's fine for now since it's the only thing we support so far.
    tone_map: Box<dyn ToneMap>,
}

impl ScreenBuffer {
    pub fn new(
        image: Image,
        out: DataWriter<ImageBufferEvent>,
        tone_map: Box<dyn ToneMap>,
    ) -> Self {
        let mut bus = DataBus::<ImageBufferEvent>::new("ScreenBuffer");
        Self {
            buffer: vec![Color::default(); image.height * image.width],
            out,
            image,
            reader: bus.get_reader(),
            bus,
            tone_map,
        }
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.reader.get_messages().and_then(|messages| {
            messages.into_iter().try_for_each(|event| {
                match event {
                    ImageBufferEvent::BufferUpdate {
                        mut rgb,
                        r,
                        c,
                        width,
                        height,
                    } => {
                        for row in 0..height {
                            for column in 0..width {
                                let buffer_index = row * width + column;
                                rgb[buffer_index] = self.tone_map.tone_map(&rgb[buffer_index]);
                                self.buffer[(r + row) * self.image.width + c + column] =
                                    rgb[buffer_index]
                            }
                        }

                        // Data processed.
                        // Pass it to the readers.
                        self.out.write(ImageBufferEvent::BufferUpdate {
                            rgb,
                            r,
                            c,
                            width,
                            height,
                        })
                    }
                }
            })
        })
    }

    pub fn get_writer(&self) -> ImageBufferWriter {
        ImageBufferWriter::new(self.bus.get_writer())
    }

    pub fn rgb(&self) -> &[Color] {
        &self.buffer
    }
}
