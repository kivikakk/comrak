#![feature(div_duration)]
#![feature(int_roundings)]
#![no_main]
use comrak::{
    markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakRenderOptions, ListStyleType,
};
use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use std::time::{Duration, Instant};

// A fuzz target which discovers quadratic parsing behaviour in comrak. This
// works differently to regular fuzz targets. Instead of trying to uncover
// memory errors, it measures the parse time of fuzz inputs to infer if the
// parse time of the input grows quadratically. If this is detected, the fuzz
// harness explicitly calls `panic`.
//
// The algorithm is roughly:
// 1. Generate a input of N bytes, parse with comrak and measure how long it
//    took to parse each byte.
// 2. Double the input size to 2*N bytes, parse with comrak and measure how
//    long it took to parse each byte.
// 3. Compare the two timings to determine if the parsing time scales
//    quadratically.
// 4. Repeat with an input of 4*N to confirm that the timings scale
//    quadratically.

/// Markdown input to the fuzzer. Based on previous quadratic parsing bugs, the
/// inputs which trigger quadratic parsing issues have common structures which is
/// represented by this enum.
#[derive(Arbitrary, Debug)]
enum Markdown {
    // <markdown> - a literal markdown string repeated, e.g.
    // foo
    // foofoo
    // foofoofoo
    // foofoofoofoo
    Markdown {
        markdown: String,
    },

    // <prefix>*N + <markdown> + <suffix>*N - a piece of markdown surrounded by a prefix/suffix pair, e.g.
    // foo
    // {foo}
    // {{foo}}
    // {{{foo}}}
    Sandwich {
        prefix: String,
        markdown: String,
        suffix: String,
    },

    // <prefix>*0 + <markdown> + <prefix>*1 + <markdown> + <prefix>*2 + <markdown> + ..., e.g.
    // foo
    // - foo
    // - - foo
    // - - - foo
    Tree {
        prefix: String,
        markdown: String,
    },
}

impl Markdown {
    /// Expand the markdown input into a string of up to size `num_bytes`
    fn render(&self, num_bytes: usize) -> String {
        let output = match self {
            Markdown::Markdown { markdown } => {
                // Repeat `markdown` but avoiding truncating the output
                let iterations = num_bytes.div_floor(markdown.len());
                markdown.repeat(iterations)
            }
            Markdown::Sandwich {
                prefix,
                markdown,
                suffix,
            } => {
                let mut output = String::with_capacity(num_bytes);
                if markdown.len() <= num_bytes {
                    let mut iterations = 0;

                    // Calculate how many "iterations" can fit into a string of size `num_bytes`
                    loop {
                        let bytes_for_iteration = prefix.len() * (iterations + 1)
                            + markdown.len()
                            + suffix.len() * (iterations + 1);
                        if bytes_for_iteration > num_bytes {
                            break;
                        }
                        iterations += 1;
                    }

                    // Place the markdown in `output`
                    for _ in 0..iterations {
                        output.push_str(&prefix)
                    }
                    output.push_str(&markdown);
                    for _ in 0..iterations {
                        output.push_str(&suffix)
                    }
                }
                output
            }
            Markdown::Tree { prefix, markdown } => {
                let mut output = String::with_capacity(num_bytes);
                if markdown.len() <= num_bytes {
                    let mut iterations = 0;

                    // Calculate how many "iterations" can fit into a string of size `num_bytes`
                    loop {
                        let bytes_for_iteration = prefix.len() * (iterations + 1) + markdown.len();
                        if bytes_for_iteration > num_bytes {
                            break;
                        }
                        iterations += 1;
                    }

                    // Place the markdown in `output`
                    for _ in 0..iterations {
                        output.push_str(&prefix)
                    }
                    output.push_str(&markdown);
                }
                output
            }
        };

        assert!(output.len() <= num_bytes);

        output
    }

    fn should_fuzz_string(s: &str) -> bool {
        if s.len() == 0 {
            // Repeating a zero-length string is useless
            return false;
        }

        if s.len() > 128 {
            // Avoid large strings
            return false;
        }

        true
    }

    /// A filter to guiding the fuzzer. The fuzzer will skip any input which fails this predicate
    fn should_fuzz(&self) -> bool {
        match self {
            Markdown::Markdown { markdown } => Markdown::should_fuzz_string(&markdown),
            Markdown::Sandwich {
                prefix,
                markdown,
                suffix,
            } => {
                Markdown::should_fuzz_string(&prefix)
                    && Markdown::should_fuzz_string(&markdown)
                    && Markdown::should_fuzz_string(&suffix)
            }
            Markdown::Tree { prefix, markdown } => {
                Markdown::should_fuzz_string(&prefix) && Markdown::should_fuzz_string(&markdown)
            }
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzComrakOptions {
    extension: FuzzComrakExtensionOptions,
    parse: FuzzComrakParseOptions,
    render: FuzzComrakRenderOptions,
}

impl FuzzComrakOptions {
    fn to_options(&self) -> ComrakOptions {
        ComrakOptions {
            extension: self.extension.to_options(),
            parse: self.parse.to_options(),
            render: self.render.to_options(),
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzComrakExtensionOptions {
    strikethrough: bool,
    tagfilter: bool,
    table: bool,
    autolink: bool,
    tasklist: bool,
    superscript: bool,
    footnotes: bool,
    description_lists: bool,
    shortcodes: bool,
}

impl FuzzComrakExtensionOptions {
    fn to_options(&self) -> ComrakExtensionOptions {
        ComrakExtensionOptions {
            strikethrough: self.strikethrough,
            tagfilter: self.tagfilter,
            table: self.table,
            autolink: self.autolink,
            tasklist: self.tasklist,
            superscript: self.superscript,
            footnotes: self.footnotes,
            description_lists: self.description_lists,
            shortcodes: self.shortcodes,
            front_matter_delimiter: None,
            header_ids: None,
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzComrakParseOptions {
    smart: bool,
    relaxed_tasklist_matching: bool,
}

impl FuzzComrakParseOptions {
    fn to_options(&self) -> ComrakParseOptions {
        ComrakParseOptions {
            smart: self.smart,
            default_info_string: None,
            relaxed_tasklist_matching: self.relaxed_tasklist_matching,
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzComrakRenderOptions {
    hardbreaks: bool,
    github_pre_lang: bool,
    full_info_string: bool,
    width: usize,
    unsafe_: bool,
    escape: bool,
    list_style: ListStyleType,
    sourcepos: bool,
}

impl FuzzComrakRenderOptions {
    fn to_options(&self) -> ComrakRenderOptions {
        ComrakRenderOptions {
            hardbreaks: self.hardbreaks,
            github_pre_lang: self.github_pre_lang,
            full_info_string: self.full_info_string,
            width: self.width,
            unsafe_: self.unsafe_,
            escape: self.escape,
            list_style: self.list_style,
            sourcepos: self.sourcepos,
        }
    }
}

/// The input to the fuzzer, which is a combination of parser options and
/// markdown input. This allows us to fuzz the markdown input with different
/// parsing options.
#[derive(Arbitrary, Debug)]
struct Input {
    options: FuzzComrakOptions,
    markdown: Markdown,
}

fn fuzz_one_input(input: &Input, num_bytes: usize) -> (usize, Duration, f64) {
    let markdown = input.markdown.render(num_bytes);

    // `should_fuzz` will guarantee that we generate non-empty inputs
    assert!(markdown.len() > 1);

    let now = Instant::now();
    {
        let _ = markdown_to_html(&markdown, &input.options.to_options());
    }
    let duration = now.elapsed();

    if DEBUG {
        println!(
            "do_one: {} bytes, duration = {:?}",
            markdown.len(),
            duration
        );
    }

    let byte_length = markdown.len();
    let duration_per_byte = duration.as_secs_f64() / (markdown.len() as f64);

    (
        byte_length,
        duration,
        duration_per_byte
    )
}

/// The maximum number of steps to run in the main fuzzing loop below.
/// Increasing the number of steps will decrease positives at the expense of
/// longer running times.
const MAX_STEPS: usize = 3;

/// The minimum ratio between steps that we consider to be quadratic runtime.
/// If the first step of N bytes executes in X seconds, and the second step of
/// M bytes executes in Y seconds then the ratio is:
///   (Y/M) / (X/N)
/// For example, if 200 bytes is parsed in 1 second and 400 bytes is parsed in
/// 2 seconds, then the ratio would be:
///
///   (400/2) / (200/1) = 1
///
/// which implies that the parsing time scales linearly.
///
/// In reality, there are fixed startup costs for each parsing run and an
/// amount of jitter for small inputs. A ratio would of 2.0 would be quadratic
/// (e.g. doubling the input size, quadruples the runtime). We use a value of
/// 2.5 by default to avoid false positives.
const MIN_RATIO: f64 = 2.5f64;

/// Set to `true` to enable extra debugging output
const DEBUG: bool = false;

fuzz_target!(|input: Input| {
    if !input.markdown.should_fuzz() {
        return;
    }

    if DEBUG {
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!("input.markdown = {:?}", input.markdown);
    }

    // Expand fuzz input to this size. This value was chosen arbitrarily
    let base_num_bytes = 1024;

    let (byte_length, duration, duration_per_byte) = fuzz_one_input(&input, base_num_bytes);

    if DEBUG {
        println!(
            "benchmark: byte_length={:?} duration={:?} duration_per_byte={:?}",
            byte_length, duration, duration_per_byte
        );
    }

    let mut duration_per_bytes = Vec::with_capacity(MAX_STEPS);
    let mut byte_lengths = Vec::with_capacity(MAX_STEPS);

    duration_per_bytes.push(duration_per_byte);
    byte_lengths.push(byte_length);

    let mut last_duration = duration_per_byte;
    let mut num_bytes = base_num_bytes;

    for i in 0..MAX_STEPS {
        // Double size of input buffer...
        num_bytes *= 2;
        let (byte_length, _, duration_per_byte) = fuzz_one_input(&input, num_bytes);

        let ratio = duration_per_byte / last_duration;
        if DEBUG {
            println!(
                "loop {}: duration_per_byte={:?} ratio={:?}",
                i, duration_per_byte, ratio
            );
        }
        // ... and check that the runtime-per-byte more than doubled, which
        // implies that the runtime-per-byte growth is superlinear
        if ratio < MIN_RATIO {
            return;
        }

        byte_lengths.push(byte_length);
        duration_per_bytes.push(duration_per_byte);
        last_duration = duration_per_byte
    }

    println!(
        "duration_per_bytes = {:?}, byte_lengths = {:?}",
        duration_per_bytes, byte_lengths
    );
    // This is printed by default when the crash is first found but not when
    // reproducing the crash, which is annoying. Explicitly print it
    println!("{:#?}", input);
    panic!()
});
