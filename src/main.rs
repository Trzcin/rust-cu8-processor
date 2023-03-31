use std::{f32::consts::{E, PI}, fs::File, io::{BufReader, Read, ErrorKind, Write, self}};
use num_complex::{Complex, ComplexFloat};

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

const BUFFER_SIZE: usize = 640;
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
        write!(&mut output, "{}", 0x2e736e64 as u32)?;
        write!(&mut output, "{}", 24 as u32)?;
        write!(&mut output, "{}", 0xffffffff as u32)?;
        write!(&mut output, "{}", 6 as u32)?;
        write!(&mut output, "{}", 48000 as u32)?;
        write!(&mut output, "{}", 1 as u32)?;
        Ok(output)
    }

    fn add_node(&mut self, node: Box<dyn ProcessingNode>) {
        self.nodes.push(node);
    }

    fn start(mut self) {
        let input_file = File::open(self.input_path).expect("cannot open input");       
        let mut reader = BufReader::new(input_file);
        let output_file = File::create(self.output_path).expect("cannot create output");
        let mut output_file = FmProcessor::prepare_output(output_file).expect("cannot write header");
        let mut buffer = [0; BUFFER_SIZE];
        loop {
            match reader.read(&mut buffer) {
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
                Ok(0) => break,
                Ok(_) => {
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
                                write!(&mut output_file, "{}", val).expect("cannot write to output");
                            }
                        }
                    }
                }
            }
        }
        
    }
}

fn main() {
    let mut processor = FmProcessor::new("input", "output");

    processor.add_node(Box::new(FmShifter(-0.0906250)));
    processor.add_node(Box::new(Decimator(5)));
    processor.add_node(Box::new(FmDemodulator));
    processor.add_node(Box::new(Decimator(8)));

    processor.start();
}
