use std::{fs::File, io::{BufReader, Read, ErrorKind}};
use num_complex::Complex;

trait ComplexSocket {
    fn process(&self, data: Vec<Complex<f32>>);
}
trait RealSocket {
    fn process(&self, data: Vec<f32>);
}

const BUFFER_SIZE: usize = 512;
struct CU8Reader<'a> {
    file_name: &'static str,
    connected: Option<&'a dyn ComplexSocket>
}
impl<'a> CU8Reader<'a> {
    fn from_file(path: &'static str) -> CU8Reader {
        CU8Reader { file_name: path, connected: None }
    }

    fn connect(&mut self, socket: &'a dyn ComplexSocket) {
        self.connected = Some(socket);
    }

    fn read(&self) {
        let file = File::open(self.file_name).expect("could not open cu8 file");
        let mut reader = BufReader::new(file);
        let mut buffer = [0; BUFFER_SIZE];
        loop {
            match reader.read(&mut buffer) {
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
                Ok(0) => break,
                Ok(_) => {
                    let output = buffer.chunks(2).map(|pair| {
                        Complex::new(
                            (pair[0] as f32 - 127.5) / 127.5,
                            (pair[1] as f32 - 127.5) / 127.5
                        )
                    }).collect();

                    match self.connected {
                        None => continue,
                        Some(socket) => socket.process(output)
                    }
                }
            }
        }
    }
}

struct Printer;
impl ComplexSocket for Printer {
    fn process(&self, data: Vec<Complex<f32>>) {
        for num in data {
            println!("{}", num);
        }
    }
}
impl RealSocket for Printer {
    fn process(&self, data: Vec<f32>) {
        for num in data {
            println!("{}", num);
        }
    }
}

fn main() {
    let mut reader = CU8Reader::from_file("some/path/file.cu8");
    let printer = Printer;
    reader.connect(&printer);
    reader.read();
}
