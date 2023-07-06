use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    let output = Command::new("rg")
        .arg("--column")
        .arg("--heading")
        .arg("--color=never")
        .arg("crs\\.execute")
        .arg("test")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to capture rg output"))?;

    let reader = BufReader::new(output);

    for line in reader.lines() {
        println!("{}", line?);
    }

    Ok(())
}
