# Rust CU8 Processor

Outputs sampled and processed data from a raw .cu8 file into an .au file in the `snd` format.

## Building

```
cargo build --release
```

## Usage

```
cd target
./rust-cu8-processor path-to-input.cu8 path-to-output.au
```

## Processing nodes

`FmProcessor` is the main object which reads data from the .cu8 file and later outputs it to the .au file.
Before outputting, the data goes through multiple chained processing nodes. These nodes include:

- `Decimator(size)` - groups samples by `size` and outputs an avarage value
- `FmShifter(p)` - moves the spectrum of the singal to match the expected receiver frequency
- `FmDemodulator` - demodulates the FM signal by outputing a real number from a complex signal
