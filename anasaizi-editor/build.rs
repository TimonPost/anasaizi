use std::{
    env,
    env::var,
    ffi::OsStr,
    fs,
    fs::remove_file,
    io::Result,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn main() {
    compile_shaders();
}

fn compile_shaders() {
    println!("Compiling shaders");

    let shader_dir_path = get_shader_source_dir_path();

    fs::read_dir(shader_dir_path.clone())
        .unwrap()
        .map(Result::unwrap)
        .filter(|dir| dir.file_type().unwrap().is_file())
        .filter(|dir| dir.path().extension() != Some(OsStr::new("spv")))
        .for_each(|dir| {
            let path = dir.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let output_name = format!("build/{}.spv", &name);
            let debug_output_name =
                format!("../../target/debug/assets/shaders/build/{}.spv", &name);

            println!("{}", debug_output_name);

            println!("Found file {:?}.\nCompiling...", path.as_os_str());

            remove_file(debug_output_name.clone());
            remove_file(output_name.clone());

            let result = Command::new("glslc")
                .current_dir(&shader_dir_path)
                .arg(&path)
                .arg("-o")
                .arg(output_name.clone())
                .output();

            handle_program_result(result);

            let result = Command::new("glslc")
                .current_dir(&shader_dir_path)
                .arg(&path)
                .arg("-o")
                .arg(debug_output_name)
                .output();

            handle_program_result(result);
        })
}

fn get_shader_source_dir_path() -> PathBuf {
    let path = get_root_path().join("assets").join("shaders");
    println!("Shader source directory: {:?}", path.as_os_str());
    path
}

fn get_root_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn handle_program_result(result: Result<Output>) {
    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Shader compilation succedeed.");
                print!(
                    "stdout: {}",
                    String::from_utf8(output.stdout)
                        .unwrap_or("Failed to print program stdout".to_string())
                );
            } else {
                eprintln!("Shader compilation failed. Status: {}", output.status);
                eprint!(
                    "stdout: {}",
                    String::from_utf8(output.stdout)
                        .unwrap_or("Failed to print program stdout".to_string())
                );
                eprint!(
                    "stderr: {}",
                    String::from_utf8(output.stderr)
                        .unwrap_or("Failed to print program stderr".to_string())
                );
                panic!("Shader compilation failed. Status: {}", output.status);
            }
        }
        Err(error) => {
            panic!("Failed to compile shader. Cause: {}", error);
        }
    }
}
