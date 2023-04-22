use std::{f32::consts::{E, PI}, fs::File,
   io::{BufReader, Read, ErrorKind, Write, self, BufWriter}, time::Instant, env};
use num_complex::Complex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut processor = FmProcessor::new(
        args.get(1).expect("Please provide input and output paths as arguments").to_owned(),
        args.get(2).expect("Please provide input and output paths as arguments").to_owned()
    );

    processor.add_node(Box::new(FmShifter(-0.0906250)));
    processor.add_node(Box::new(Decimator(5)));
    processor.add_node(Box::new(FmDemodulator));
    processor.add_node(Box::new(Decimator(8)));

    processor.start();
}

trait ProcessingNode {
    fn process(&self, data: Vec<Complex<f32>>) -> Vec<Complex<f32>>;
}

struct Decimator(usize);
impl ProcessingNode for Decimator {
    fn process(&self, data: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        data.chunks(self.0)
            .map(|chunk|
                chunk.into_iter().sum::<Complex<f32>>() / Complex::new(chunk.len() as f32, 0.0)
            ).collect()
    }
}

struct FmShifter(f32);
impl ProcessingNode for FmShifter {
    fn process(&self, data: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        data.into_iter().enumerate().map(|(n, val)|
            val * Complex::new(E,0.0).powc(Complex::new(0.0, 2.0 * PI * self.0 * n as f32))
        ).collect()
    }
}

struct FmDemodulator;
impl ProcessingNode for FmDemodulator {
    fn process(&self, data: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        data.windows(2).map(|chunk| {
            Complex::new((chunk[1] * chunk[0].conj()).arg() / PI, 0.0)
        }).collect()
    }
}

const BUFFER_SIZE: usize = 640000;
struct FmProcessor {
    input_path: String,
    output_path: String,
    data: Vec<Complex<f32>>,
    nodes: Vec<Box<dyn ProcessingNode>>
}
impl FmProcessor {
    fn new(input: String, output: String) -> FmProcessor {
        FmProcessor { input_path: input, output_path: output,
            data: vec![], nodes: vec![] }
    }

    fn prepare_output(mut output: BufWriter<File>) -> Result<BufWriter<File>, io::Error> {
        let header_params: [u32; 6] = [0x2e736e64, 24, 0xffffffff, 6, 48000, 1];
        for param in header_params {
            output.write(&param.to_be_bytes())?;
        }
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
        let writer = BufWriter::new(output_file);
        let mut writer = FmProcessor::prepare_output(writer).expect("cannot write header");
        let mut buffer = [0; BUFFER_SIZE];
        let t1 = Instant::now();
        let mut bytes_read = 0;
        loop {
            match reader.read(&mut buffer) {
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
                Ok(0) => break,
                Ok(n) => {
                    self.data = buffer
                        .chunks(2).map(|pair|
                            Complex::new(
                                (pair[0] as f32 - 127.5) / 127.5,
                                (pair[1] as f32 - 127.5) / 127.5
                            )
                        ).collect();

                    for node in &self.nodes {
                        self.data = node.process(self.data);
                    }

                    for val in self.data {
                        writer.write(&val.re.to_be_bytes()).expect("cannot write to output");
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
