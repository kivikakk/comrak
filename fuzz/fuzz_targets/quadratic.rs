#![feature(div_duration)]
#![feature(int_roundings)]
#![no_main]
use comrak::{
    markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html, ExtensionOptions,
    ListStyleType, Options, ParseOptions, RenderOptions,
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
struct FuzzOptions {
    extension: FuzzExtensionOptions,
    parse: FuzzParseOptions,
    render: FuzzRenderOptions,
}

impl FuzzOptions {
    fn to_options(&self) -> Options {
        Options {
            extension: self.extension.to_options(),
            parse: self.parse.to_options(),
            render: self.render.to_options(),
        }
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzExtensionOptions {
    strikethrough: bool,
    tagfilter: bool,
    table: bool,
    autolink: bool,
    tasklist: bool,
    superscript: bool,
    footnotes: bool,
    description_lists: bool,
    multiline_block_quotes: bool,
    math_dollars: bool,
    math_code: bool,
    shortcodes: bool,
    wikilinks_title_after_pipe: bool,
    wikilinks_title_before_pipe: bool,
}

impl FuzzExtensionOptions {
    fn to_options(&self) -> ExtensionOptions {
        let mut extension = ExtensionOptions::default();
        extension.strikethrough = self.strikethrough;
        extension.tagfilter = self.tagfilter;
        extension.table = self.table;
        extension.autolink = self.autolink;
        extension.tasklist = self.tasklist;
        extension.superscript = self.superscript;
        extension.footnotes = self.footnotes;
        extension.description_lists = self.description_lists;
        extension.multiline_block_quotes = self.multiline_block_quotes;
        extension.math_dollars = self.math_dollars;
        extension.math_code = self.math_code;
        extension.shortcodes = self.shortcodes;
        extension.wikilinks_title_after_pipe = self.wikilinks_title_after_pipe;
        extension.wikilinks_title_before_pipe = self.wikilinks_title_before_pipe;
        extension.front_matter_delimiter = None;
        extension.header_ids = None;
        extension
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzParseOptions {
    smart: bool,
    relaxed_tasklist_matching: bool,
    relaxed_autolinks: bool,
}

impl FuzzParseOptions {
    fn to_options(&self) -> ParseOptions {
        let mut parse = ParseOptions::default();
        parse.smart = self.smart;
        parse.default_info_string = None;
        parse.relaxed_tasklist_matching = self.relaxed_tasklist_matching;
        parse.relaxed_autolinks = self.relaxed_autolinks;
        parse
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzRenderOptions {
    hardbreaks: bool,
    github_pre_lang: bool,
    full_info_string: bool,
    width: usize,
    unsafe_: bool,
    escape: bool,
    list_style: ListStyleType,
    sourcepos: bool,
    escaped_char_spans: bool,
}

impl FuzzRenderOptions {
    fn to_options(&self) -> RenderOptions {
        let mut render = RenderOptions::default();
        render.hardbreaks = self.hardbreaks;
        render.github_pre_lang = self.github_pre_lang;
        render.full_info_string = self.full_info_string;
        render.width = self.width;
        render.unsafe_ = self.unsafe_;
        render.escape = self.escape;
        render.list_style = self.list_style;
        render.sourcepos = self.sourcepos;
        render.escaped_char_spans = self.escaped_char_spans;
        render
    }
}

/// The input to the fuzzer, which is a combination of parser options and
/// markdown input. This allows us to fuzz the markdown input with different
/// parsing options.
#[derive(Arbitrary, Debug)]
struct Input {
    options: FuzzOptions,
    markdown: Markdown,
}

fn fuzz_one_input(input: &Input, num_bytes: usize) -> (usize, Duration, f64) {
    let markdown = input.markdown.render(num_bytes);

    // `should_fuzz` will guarantee that we generate non-empty inputs
    assert!(markdown.len() > 1);

    let now = Instant::now();
    {
        let _ = markdown_to_html(&markdown, &input.options.to_options());
        let _ = markdown_to_commonmark(&markdown, &input.options.to_options());
        let _ = markdown_to_commonmark_xml(&markdown, &input.options.to_options());
    }

    let duration = now.elapsed();
    let byte_length = markdown.len() * 3;
    let duration_per_byte = duration.as_secs_f64() / (byte_length as f64);

    if DEBUG {
        println!("do_one: {} bytes, duration = {:?}", byte_length, duration);
    }

    (byte_length, duration, duration_per_byte)
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
