use num_complex::Complex;
#[allow(dead_code)]

enum FmData {
    ComplexData(Vec<Complex<f32>>),
    RealData(Vec<f32>)
}

trait ProcessingNode {
    fn process(&self, data: FmData) -> FmData;
}

struct Decimator(u8);
impl ProcessingNode for Decimator {
    fn process(&self, data: FmData) -> FmData {
        match data {
            FmData::ComplexData(data) => todo!(),
            FmData::RealData(data) => todo!()
        }
    }
}

struct FmShifter(f32);
impl ProcessingNode for FmShifter {
    fn process(&self, data: FmData) -> FmData {
        match data {
            FmData::RealData(_) => panic!("FmShifter expected complex data, received real data"),
            FmData::ComplexData(data) => todo!()
        }
    }
}

struct FmDemodulator;
impl ProcessingNode for FmDemodulator {
    fn process(&self, data: FmData) -> FmData {
        match data {
            FmData::RealData(_) => panic!("FmDemodulator expected complex data, received real data"),
            FmData::ComplexData(data) => todo!()
        }
    }
}

#[allow(dead_code)]
struct FmProcessor {
    input_file: &'static str,
    output_file: &'static str,
    nodes: Vec<Box<dyn ProcessingNode>>
}
#[allow(dead_code)]
impl FmProcessor {
    fn new(input: &'static str, output: &'static str) -> FmProcessor {
        FmProcessor { input_file: input, output_file: output, nodes: vec![] }
    }

    fn add_node(&mut self, node: Box<dyn ProcessingNode>) {
        self.nodes.push(node);
    }

    fn start() {
        todo!();
    }
}

fn main() {

}
