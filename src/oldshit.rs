#![allow(dead_code)]
use std::path::PathBuf;

struct FileMatches {
    path: PathBuf,
    matches: Vec<Match>,
}

impl From<&str> for FileMatches {
    fn from(value: &str) -> Self {
        let heading = value.lines().next().unwrap().trim();
        let matches = value
            .lines()
            .skip(1)
            .map(|line| Match::from(line))
            .collect();

        Self {
            path: PathBuf::from(heading),
            matches,
        }
    }
}

struct Match {
    lnum: usize,
    col_start: usize,
    col_end: usize,
}

impl From<&str> for Match {
    fn from(value: &str) -> Self {
        let mut line = value.trim().split(":");
        let lnum = line.next().unwrap().parse::<usize>().unwrap();
        let col = line.next().unwrap().parse::<usize>().unwrap();
        Self {
            lnum,
            col_start: col,
            col_end: col,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn match_from() {
        let line = r#"3:1:crs.execute("SELECT * FROM foo")"#;
        let m = Match::from(line);
        assert_eq!(m.lnum, 3);
        assert_eq!(m.col_start, 1);
        assert_eq!(m.col_end, 1);
    }

    #[test]
    fn filematches_from() {
        let lines = r#"test/a.py
3:1:crs.execute("SELECT * FROM foo")
5:1:crs.execute("SELECT * FROM foo")
6:1:crs.execute("SELECT * FROM foo")
7:1:crs.execute("SELECT * FROM foo")
8:1:crs.execute("SELECT * FROM foo")"#;

        let fm = FileMatches::from(lines);
        assert_eq!(fm.path.to_str().unwrap(), "test/a.py");
        assert_eq!(fm.matches.len(), 5);
    }
}

