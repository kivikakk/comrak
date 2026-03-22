use comrak::{Arena, Options, parse_document, html};
use comrak::options::Plugins;
use std::io::Read;
use std::time::Instant;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    
    let options = Options::default();
    let plugins = Plugins::default();
    
    // Warmup
    for _ in 0..3 {
        let arena = Arena::new();
        let root = parse_document(&arena, &input, &options);
        let mut output = String::with_capacity(input.len() + input.len() / 4);
        html::format_document_to_string(root, &options, &mut output, &plugins).unwrap();
    }
    
    let iterations = 15;
    let mut times = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let arena = Arena::new();
        let start = Instant::now();
        let root = parse_document(&arena, &input, &options);
        let mut output = String::with_capacity(input.len() + input.len() / 4);
        html::format_document_to_string(root, &options, &mut output, &plugins).unwrap();
        let elapsed = start.elapsed();
        times.push(elapsed);
        std::hint::black_box(&output);
    }
    
    times.sort();
    let median = times[iterations / 2];
    let min = times[0];
    let mean: std::time::Duration = times.iter().sum::<std::time::Duration>() / iterations as u32;
    
    let median_ms = median.as_secs_f64() * 1000.0;
    let min_ms = min.as_secs_f64() * 1000.0;
    let mean_ms = mean.as_secs_f64() * 1000.0;
    
    eprintln!("median: {:.2}ms, mean: {:.2}ms, min: {:.2}ms ({} iterations)", 
              median_ms, mean_ms, min_ms, iterations);
    println!("METRIC median_ms={:.2}", median_ms);
    println!("METRIC mean_ms={:.2}", mean_ms);
    println!("METRIC min_ms={:.2}", min_ms);
}
