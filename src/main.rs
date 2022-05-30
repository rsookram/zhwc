use anyhow::{bail, Result};
use jieba_rs::Jieba;
use pico_args::Arguments;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

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
    let excludes = Arc::new(excludes);

    let jieba = Arc::new(Jieba::new());

    let mut handles = Vec::new();

    for path in paths {
        // TODO: Limit parallelism
        let jieba = Arc::clone(&jieba);
        let excludes = Arc::clone(&excludes);

        handles.push(thread::spawn(move || {
            fs::read_to_string(&path).map(|text| {
                let mut counts = HashMap::<_, u32>::with_capacity(1024);

                let words = jieba
                    .cut(&text, true)
                    .into_iter()
                    .filter(|w| should_count(&excludes, w));

                for w in words {
                    *counts.entry(w.to_string()).or_insert(0) += 1;
                }

                counts
            })
        }));
    }

    let mut counts = HashMap::<_, u32>::with_capacity(8 * 1024);

    for handle in handles {
        let words = match handle.join() {
            Ok(r) => r?,
            Err(e) => bail!("{e:?}"),
        };

        for (w, count) in words {
            *counts.entry(w).or_insert(0) += count;
        }
    }

    let mut stdout = BufWriter::with_capacity(64 * 1024, std::io::stdout().lock());

    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by_cached_key(|(w, c)| (std::cmp::Reverse(*c), w.to_string()));

    for (word, count) in counts {
        writeln!(stdout, "{word} {count}")?;
    }

    stdout.flush()?;

    Ok(())
}

fn should_count(excludes: &HashSet<String>, word: &str) -> bool {
    !excludes.contains(word) && word.chars().any(|c| c >= '一' && c <= '\u{9fff}')
}
