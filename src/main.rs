use anyhow::Result;
use jieba_rs::Jieba;
use std::collections::HashMap;
use std::io::BufWriter;
use std::io::Write;

fn main() -> Result<()> {
    let mut args = std::env::args_os();
    args.next(); // Skip the program name

    let jieba = Jieba::new();

    let mut counts = HashMap::<_, u32>::new();

    for path in args {
        let text = std::fs::read_to_string(&path)?;

        let words = jieba.cut(&text, true);
        for w in words {
            *counts.entry(w.to_string()).or_insert(0) += 1;
        }
    }

    let mut stdout = BufWriter::with_capacity(64 * 1024, std::io::stdout().lock());

    for (word, count) in counts {
        writeln!(stdout, "{word} {count}")?;
    }

    stdout.flush()?;

    Ok(())
}
