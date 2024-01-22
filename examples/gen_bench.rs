use std::env;

fn main() {
    let msg = "First cli arg is count";
    let n: usize = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("10000"))
        .parse()
        .expect(msg);

    println!("Numbers = {{ value, square, rand");

    for i in 0..n {
        let c = i % CHARS.len(); // this slice is fine cause its all ascii
        println!("{i}, {}, '{}'", i * i, &CHARS[c..c + 1].repeat(15))
    }

    println!("}}");
    println!(
        "pi value (σ ((value < 999) AND (rand='aaaaaaaaaaaaaaa')) (pi value, rand (σ (square > (value + 1000)) (pi rand, value, square (Numbers)))))"
    );
}

const CHARS: &str = "qwertyuiopasdfghjklzxcvbnm1234567890";
