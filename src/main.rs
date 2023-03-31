use std::{f32::consts::{E, PI}, fs::File, io::{BufReader, Read, ErrorKind, Write, self}, time::Instant};
use num_complex::{Complex, ComplexFloat};

fn main() {
    let mut processor = FmProcessor::new(
        "/home/trzcinkde/Documents/Programming/rust/fm-audio-proccess/data/fm1_99M726_1M92.cu8",
        "/home/trzcinkde/Documents/Programming/rust/fm-audio-proccess/data/out.au"
    );

    processor.add_node(Box::new(FmShifter(-0.0906250)));
    processor.add_node(Box::new(Decimator(5)));
    processor.add_node(Box::new(FmDemodulator));
    processor.add_node(Box::new(Decimator(8)));

    processor.start();
}

enum FmData {
    ComplexData(Vec<Complex<f32>>),
    RealData(Vec<f32>)
}

trait ProcessingNode {
    fn process(&self, data: FmData) -> FmData;
}

struct Decimator(usize);
impl ProcessingNode for Decimator {
    fn process(&self, data: FmData) -> FmData {
        match data {
            FmData::ComplexData(data) => {
                FmData::ComplexData(
                    data.chunks(self.0)
                        .map(|chunk| 
                            chunk.into_iter().sum::<Complex<f32>>() / Complex::new(chunk.len() as f32, 0.0)
                        ).collect()
                )
            },
            FmData::RealData(data) => {
                FmData::RealData(
                    data.chunks(self.0)
                        .map(|chunk| 
                            chunk.into_iter().sum::<f32>() / chunk.len() as f32
                        ).collect()
                )
            }
        }
    }
}

struct FmShifter(f32);
impl ProcessingNode for FmShifter {
    fn process(&self, data: FmData) -> FmData {
        match data {
            FmData::RealData(_) => panic!("FmShifter expected complex data, received real data"),
            FmData::ComplexData(data) => {
                FmData::ComplexData(
                    data.into_iter().enumerate().map(|(n, val)|
                        val * Complex::new(E,0.0).powc(Complex::new(0.0, 2.0 * PI * self.0 * n as f32))
                    ).collect()
                )
            }
        }
    }
}

struct FmDemodulator;
impl ProcessingNode for FmDemodulator {
    fn process(&self, data: FmData) -> FmData {
        match data {
            FmData::RealData(_) => panic!("FmDemodulator expected complex data, received real data"),
            FmData::ComplexData(data) => {
                FmData::RealData(
                    data.windows(2).map(|chunk| {
                        (chunk[1] * chunk[0].conj()).arg() / PI
                    }).collect()
                )
            }
        }
    }
}

const BUFFER_SIZE: usize = 640000;
struct FmProcessor {
    input_path: &'static str,
    output_path: &'static str,
    data: FmData,
    nodes: Vec<Box<dyn ProcessingNode>>
}
impl FmProcessor {
    fn new(input: &'static str, output: &'static str) -> FmProcessor {
        FmProcessor { input_path: input, output_path: output, 
            data: FmData::ComplexData(vec![]), nodes: vec![] }
    }

    fn prepare_output(mut output: File) -> Result<File, io::Error> {
        output.write(&(0x2e736e64 as u32).to_le_bytes())?;
        output.write(&(24 as u32).to_le_bytes())?;
        output.write(&(0xffffffff as u32).to_le_bytes())?;
        output.write(&(6 as u32).to_le_bytes())?;
        output.write(&(48000 as u32).to_le_bytes())?;
        output.write(&(1 as u32).to_le_bytes())?;
        Ok(output)
    }

    fn add_node(&mut self, node: Box<dyn ProcessingNode>) {
        self.nodes.push(node);
    }

    fn start(mut self) {
        let input_file = File::open(self.input_path).expect("cannot open input");       
        let mut reader = BufReader::new(input_file);
        let output_file = File::options()
            .create(true)
            .append(true)
            .open(self.output_path).expect("cannot prepare output file");
        let mut output_file = FmProcessor::prepare_output(output_file).expect("cannot write header");
        let mut buffer = [0; BUFFER_SIZE];
        let t1 = Instant::now();
        let mut bytes_read = 0;
        loop {
            match reader.read(&mut buffer) {
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
                Ok(0) => break,
                Ok(n) => {
                    self.data = FmData::ComplexData(
                        buffer.chunks(2).map(|pair| 
                            Complex::new(
                                (pair[0] as f32 - 127.5) / 127.5, 
                                (pair[1] as f32 - 127.5) / 127.5
                            )
                        ).collect()
                    );

                    for node in &self.nodes {
                        self.data = node.process(self.data);
                    }

                    match self.data {
                        FmData::ComplexData(_) => panic!("Trying to save complex data"),
                        FmData::RealData(data) => {
                            for val in data {
                                output_file.write(&val.to_le_bytes()).expect("cannot write to output");
                            }
                        }
                    }
                    bytes_read += n;
                }
            }
        }

        let elapsed = t1.elapsed().as_secs_f32();
        let speed = (bytes_read as f32) / 1_048_576f32 / elapsed;
        println!("Parsed {} bytes in {:.2} seconds at {:.2} MB/s", bytes_read, elapsed, speed);
    }
}
