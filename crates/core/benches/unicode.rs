//! Unicode width calculation performance benchmarks.

#![allow(clippy::semicolon_if_nothing_returned)]

use criterion::{Criterion, criterion_group, criterion_main};
use opentui::unicode::{
    WidthMethod, display_width, display_width_char, display_width_char_with_method,
    display_width_with_method, grapheme_info, graphemes, is_ascii_only,
};
use opentui::{OptimizedBuffer, Style};
use opentui_core as opentui;
use std::hint::black_box;

fn width_ascii(c: &mut Criterion) {
    let ascii_text = "Hello, World! This is a test string.";

    c.bench_function("display_width_ascii_short", |b| {
        b.iter(|| display_width(black_box(ascii_text)));
    });

    let ascii_long = "x".repeat(1000);
    c.bench_function("display_width_ascii_1000", |b| {
        b.iter(|| display_width(black_box(&ascii_long)));
    });
}

fn width_unicode(c: &mut Criterion) {
    // Mixed ASCII and wide characters
    let mixed = "Hello, 世界! こんにちは";

    c.bench_function("display_width_mixed", |b| {
        b.iter(|| display_width(black_box(mixed)));
    });

    // All wide characters
    let cjk = "中文测试字符串这是一个很长的中文文本";

    c.bench_function("display_width_cjk", |b| {
        b.iter(|| display_width(black_box(cjk)));
    });

    // Emoji
    let emoji = "🎉🎊🎁🎂🎈🎄🎃🎇🎆✨";

    c.bench_function("display_width_emoji", |b| {
        b.iter(|| display_width(black_box(emoji)));
    });

    // Complex graphemes (combining characters)
    let combining = "é̃ñ café naïve";

    c.bench_function("display_width_combining", |b| {
        b.iter(|| display_width(black_box(combining)));
    });
}

fn width_char(c: &mut Criterion) {
    c.bench_function("display_width_char_ascii", |b| {
        b.iter(|| display_width_char(black_box('A')));
    });

    c.bench_function("display_width_char_cjk", |b| {
        b.iter(|| display_width_char(black_box('中')));
    });

    c.bench_function("display_width_char_emoji", |b| {
        b.iter(|| display_width_char(black_box('🎉')));
    });
}

fn width_methods(c: &mut Criterion) {
    let mixed = "Hello, 世界! 🎉";

    c.bench_function("display_width_wcwidth", |b| {
        b.iter(|| display_width_with_method(black_box(mixed), WidthMethod::WcWidth));
    });

    c.bench_function("display_width_unicode", |b| {
        b.iter(|| display_width_with_method(black_box(mixed), WidthMethod::Unicode));
    });

    c.bench_function("display_width_char_wcwidth", |b| {
        b.iter(|| display_width_char_with_method(black_box('世'), WidthMethod::WcWidth));
    });

    c.bench_function("display_width_char_unicode", |b| {
        b.iter(|| display_width_char_with_method(black_box('世'), WidthMethod::Unicode));
    });
}

fn grapheme_operations(c: &mut Criterion) {
    let text = "Hello, 世界! こんにちは 🎉";

    c.bench_function("graphemes_iterate", |b| {
        b.iter(|| graphemes(black_box(text)).count());
    });

    c.bench_function("grapheme_info_collect", |b| {
        b.iter(|| grapheme_info(black_box(text), 4, WidthMethod::WcWidth));
    });

    let long_text = "Hello, 世界! ".repeat(100);
    c.bench_function("graphemes_long", |b| {
        b.iter(|| graphemes(black_box(&long_text)).count());
    });
}

fn ascii_detection(c: &mut Criterion) {
    let ascii = "Hello, World! This is all ASCII text.";
    let unicode = "Hello, 世界!";

    c.bench_function("is_ascii_only_true", |b| {
        b.iter(|| is_ascii_only(black_box(ascii)));
    });

    c.bench_function("is_ascii_only_false", |b| {
        b.iter(|| is_ascii_only(black_box(unicode)));
    });

    let long_ascii = "x".repeat(1000);
    c.bench_function("is_ascii_only_long", |b| {
        b.iter(|| is_ascii_only(black_box(&long_ascii)));
    });
}

fn complex_graphemes(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_graphemes");

    // ZWJ family emoji (👨‍👩‍👧‍👦 = 7 codepoints, 1 grapheme)
    let family = "👨‍👩‍👧‍👦";
    group.bench_function("width_family_emoji", |b| {
        b.iter(|| display_width(black_box(family)));
    });

    // Flag emoji (🏳️‍🌈 = 4 codepoints, 1 grapheme)
    let flag = "🏳️‍🌈";
    group.bench_function("width_flag_emoji", |b| {
        b.iter(|| display_width(black_box(flag)));
    });

    // Person with skin tone and profession (👩🏽‍💻)
    let person = "👩🏽‍💻";
    group.bench_function("width_person_emoji", |b| {
        b.iter(|| display_width(black_box(person)));
    });

    // Many ZWJ sequences
    let many_zwj = "👨‍👩‍👧‍👦👩🏽‍💻🏳️‍🌈👨‍🍳".repeat(25);
    group.bench_function("width_100_zwj", |b| {
        b.iter(|| display_width(black_box(&many_zwj)));
    });

    // Grapheme iteration over complex emoji
    group.bench_function("graphemes_100_zwj", |b| {
        b.iter(|| graphemes(black_box(&many_zwj)).count());
    });

    // Long CJK text (common in Asian terminals)
    let cjk_long = "日本語テキスト中文文本한국어".repeat(100);
    group.bench_function("width_cjk_3000_chars", |b| {
        b.iter(|| display_width(black_box(&cjk_long)));
    });

    group.bench_function("graphemes_cjk_3000_chars", |b| {
        b.iter(|| graphemes(black_box(&cjk_long)).count());
    });

    group.finish();
}

fn draw_unicode_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_unicode_text");

    // ASCII text
    let ascii_80 = "x".repeat(80);
    group.bench_function("ascii_80_chars", |b| {
        let mut buffer = OptimizedBuffer::new(80, 24);
        b.iter(|| buffer.draw_text(0, 0, black_box(&ascii_80), Style::default()));
    });

    // CJK text (40 wide chars = 80 columns)
    let cjk_40 = "日".repeat(40);
    group.bench_function("cjk_40_chars", |b| {
        let mut buffer = OptimizedBuffer::new(80, 24);
        b.iter(|| buffer.draw_text(0, 0, black_box(&cjk_40), Style::default()));
    });

    // Mixed content
    let mixed = "Hello 日本語 World 👋 こんにちは! ";
    group.bench_function("mixed_content", |b| {
        let mut buffer = OptimizedBuffer::new(80, 24);
        b.iter(|| buffer.draw_text(0, 0, black_box(mixed), Style::default()));
    });

    // Full screen of unicode (80x24)
    let line = "Hello 世界 🎉 ".repeat(5);
    group.bench_function("full_screen_mixed", |b| {
        let mut buffer = OptimizedBuffer::new(80, 24);
        b.iter(|| {
            for y in 0..24 {
                buffer.draw_text(0, y, black_box(&line), Style::default());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    width_ascii,
    width_unicode,
    width_char,
    width_methods,
    grapheme_operations,
    ascii_detection,
    complex_graphemes,
    draw_unicode_text
);
criterion_main!(benches);
