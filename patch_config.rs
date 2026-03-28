use directories::ProjectDirs;
use std::path::PathBuf;

fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "BastionCharacterSheet") {
        let dir = proj_dirs.state_dir().unwrap_or_else(|| proj_dirs.data_local_dir());
        std::fs::create_dir_all(dir).ok();
        let mut path = dir.to_path_buf();
        path.push("config.json");
        path
    } else {
        PathBuf::from("bastion_sheet_config.json")
    }
}

fn main() {
    println!("{:?}", get_config_path());
}
