use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
};

#[derive(Debug)]
struct FileNotFound(String);

impl Display for FileNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "File not found: {}", self.0)
    }
}

impl Error for FileNotFound {}

fn last_n_segments(n: u32, path: &str) -> String {
    assert_ne!(n, 0);
    let mut i = 0;
    let mut index = 0;
    while i < n {
        if let Some(j) = path.rfind("/") {
            index = j;
        }
        i += 1;
    }
    path[index + 1..].to_owned()
}

fn get_newest_save_file(console_name: &str, game_name: &str) -> Result<File, Box<dyn Error>> {
    let dir_path = format!("saves/{console_name}/{game_name}");
    let dir = fs::read_dir(dir_path.clone())?;
    dir.into_iter()
        .flat_map(|x| x)
        .filter(|f| f.metadata().map_or(false, |f| f.is_dir()))
        .max_by(|a, b| {
            a.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&b.metadata().unwrap().modified().unwrap())
        })
        .map_or(
            Err(Box::new(FileNotFound(dir_path.clone()))),
            |f| match File::open(f.path()) {
                Ok(file) => Ok(file),
                Err(_) => Err(Box::new(FileNotFound(dir_path))),
            },
        )
}

fn get_file(console_name: &str, game_name: &str, file_name: &str) -> Result<File, Box<dyn Error>> {
    let file_path = format!("saves/{console_name}/{game_name}/{file_name}.save");
    match File::open(&file_path) {
        Ok(file) => Ok(file),
        Err(_) => Err(Box::new(FileNotFound(file_path))),
    }
}

fn get_files_for_game(console_name: &str, game_name: &str) -> Vec<File> {
    let dir_path = format!("saves/{console_name}/{game_name}");
    match fs::read_dir(dir_path) {
        Ok(dir) => dir
            .filter(|f| {
                f.as_ref()
                    .map_or(false, |f| f.metadata().map_or(false, |f| f.is_file()))
            })
            .flat_map(|f| f.and_then(|f| File::open(f.path())))
            .collect(),
        Err(_) => vec![],
    }
}

fn get_dirs(console_name: &str) -> Vec<String> {
    let dir_path = format!("saves/{console_name}");
    println!("dir_path: {dir_path}");
    match fs::read_dir(dir_path) {
        Ok(dir) => dir
            .filter(|x| {
                x.as_ref()
                    .map_or(false, |f| f.metadata().map_or(false, |f| f.is_dir()))
            })
            .flat_map(|x| x.map(|x| x.path().to_str().map(|x| x.to_string())).unwrap())
            .collect(),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::filio::last_n_segments;

    use super::get_dirs;

    #[test]
    fn test_get_dirs() {
        let dirs = get_dirs("gbc");
        let dirs: Vec<_> = dirs.iter().map(|x| last_n_segments(1, x)).collect();
        assert_eq!(dirs, vec!["Pokemon-Silver"])
    }
}
