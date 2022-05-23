use anyhow::Result;
use jieba_rs::Jieba;
use pico_args::Arguments;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let excludes_path = args.opt_value_from_os_str(["-e", "--excludes"], |v| {
        Ok::<_, Infallible>(PathBuf::from(v))
    })?;

    let paths = args.finish();

    let excludes = match excludes_path {
        Some(path) => {
            let mut set = HashSet::with_capacity(1024);
            for word in fs::read_to_string(&path)?.split('\n') {
                set.insert(word.to_string());
            }

            set
        }
        None => Default::default(),
    };

    let jieba = Jieba::new();

    let mut counts = HashMap::<_, u32>::with_capacity(8 * 1024);

    for path in paths {
        let text = fs::read_to_string(&path)?;

        let words = jieba.cut(&text, true);
        for w in words {
            if should_count(&excludes, w) {
                *counts.entry(w.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut stdout = BufWriter::with_capacity(64 * 1024, std::io::stdout().lock());

    for (word, count) in counts {
        writeln!(stdout, "{word} {count}")?;
    }

    stdout.flush()?;

    Ok(())
}

fn should_count(excludes: &HashSet<String>, word: &str) -> bool {
    !excludes.contains(word) && word.chars().any(|c| c >= '一' && c <= '\u{9fff}')
}
