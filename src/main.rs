use ringbuf::{RingBuffer, Consumer};
use itertools::{Itertools, Chunks};

const DICT_SIZE: usize = 0x80;
const LA_SIZE: usize = 0x10;


fn ascii<'a, I>(bytes: I) -> String
    where 
        I: IntoIterator<Item = &'a u8>
{
    let ascii_bytes: Vec<u8> = bytes.into_iter().map(|b| match b {
        b if *b >= 32 && *b <= 126 => *b,
        _ => '.' as u8,
    }).collect();
    String::from_utf8(ascii_bytes).unwrap()
}

fn print_consumer(consumer: Consumer<u8>) {
    consumer.access(|a, b| {
        let bytes = a.iter().chain(b);
        bytes
            .chunks(16)
            .into_iter()
            .map(ascii)
            .for_each(|ascii_str| {
                println!("{}", ascii_str);
            })
    });
}

fn main() {
    let dict_buf = RingBuffer::<u8>::new(DICT_SIZE);
    let (_dict_producer, dict_consumer) = dict_buf.split();

    let la_buf = RingBuffer::<u8>::new(LA_SIZE);
    let (mut la_producer, la_consumer) = la_buf.split();

    for _ in 0..LA_SIZE {
        la_producer.push(0).unwrap();
    }

    println!("Dict:");
    print_consumer(dict_consumer);

    println!("LA:");
    print_consumer(la_consumer);
}
