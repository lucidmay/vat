use std::path::PathBuf;
fn main(){
    let app_path = PathBuf::from("C:\\Program Files\\DJV\\bin");
    let new_path = format!("{};{}", app_path.display(), std::env::var("PATH").unwrap_or_default());
    let file_path = PathBuf::from("Z:\\noduro_roots\\duker\\checkmine\\component1\\render.1.jpg");


    let mut command = std::process::Command::new("djv_view");
    command.env("PATH", new_path);
    command.args(&[&file_path.display().to_string()]);
    command.spawn().unwrap();
}