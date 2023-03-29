use std::{fs::File, io::{BufReader, Read, ErrorKind}, time::Instant};

const BUFFER_SIZE: usize = 512;
fn main() {
    let reader = CU8Reader::from_file("/home/trzcinkde/Documents/Programming/rust/fm-audio-proccess/data/fm1_99M726_1M92.cu8");
    reader.run();
}

struct CU8Reader(&'static str);
impl CU8Reader {
    fn from_file(file_name: &'static str) -> CU8Reader {
        CU8Reader(file_name)
    }

    fn run(&self) {
        let file = File::open(self.0).expect("could not open cu8 file");
        let mut buf_reader = BufReader::new(file);
        let mut byte_count = 0;
        let mut buffer = [0; BUFFER_SIZE];
        let t1 = Instant::now();
        loop {
            match buf_reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    byte_count += n;
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
            }
        }
        let duration = t1.elapsed().as_secs_f64();
        let speed = (byte_count as f64) / 1_048_576f64 / duration;
        println!("Found: {} bytes in {:.2} seconds. Speed: {:.1} MB/s", byte_count, duration, speed);
    }
}
