//! This build script reads config file paths from the `UMHV_VM_CONFIGS` environment variable,
//! reads them, and then outputs them to `$(OUT_DIR)/vm_configs.rs` to be used by
//! `src/vmm/config.rs`.
//!
//! The `UMHV_VM_CONFIGS` environment variable should follow the format convention for the
//! `PATH` environment variable on the building platform, i.e., paths are separated by colons
//! (`:`) on Unix-like systems and semicolons (`;`) on Windows.
//!
//! In the generated `vm_configs.rs` file, a function `static_vm_configs` is defined that
//! returns a `Vec<&'static str>` containing the contents of the configuration files.
//!
//! If the `UMHV_VM_CONFIGS` environment variable is not set, `static_vm_configs` will call
//! the `default_static_vm_configs` function from `src/vmm/config.rs` to return the default
//! configurations.
//!
//! If the `UMHV_VM_CONFIGS` environment variable is set but the configuration files cannot be
//! read, the build script will output a `compile_error!` macro that will cause the build to
//! fail.
//!
//! This build script reruns if the `UMHV_VM_CONFIGS` environment variable changes, or if the
//! `build.rs` file changes, or if any of the files in the paths specified by `UMHV_VM_CONFIGS`
//! change.
use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use toml::Value;

/// A configuration file that has been read from disk.
struct ConfigFile {
    /// The path to the configuration file.
    pub path: OsString,
    /// The contents of the configuration file.
    pub content: String,
}

/// Gets the paths (colon-separated) from the `UMHV_VM_CONFIGS` environment variable.
///
/// Returns `None` if the environment variable is not set.
fn get_config_paths() -> Option<Vec<OsString>> {
    env::var_os("UMHV_VM_CONFIGS")
        .map(|paths| env::split_paths(&paths).map(OsString::from).collect())
}

/// Gets the paths and contents of the configuration files specified by the `UMHV_VM_CONFIGS` environment variable.
///
/// Returns a tuple of the paths and contents of the configuration files if successful, or an error message if not.
fn get_configs() -> Result<Vec<ConfigFile>, String> {
    get_config_paths()
        .map(|paths| {
            paths
                .into_iter()
                .map(|path| {
                    let path_buf = PathBuf::from(&path);
                    let content = fs::read_to_string(&path_buf).map_err(|e| {
                        format!("Failed to read file {}: {}", path_buf.display(), e)
                    })?;
                    Ok(ConfigFile { path, content })
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(vec![]))
}

/// Opens the output file for writing.
///
/// Returns the file handle.
fn open_output_file() -> fs::File {
    let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_file = output_dir.join("vm_configs.rs");

    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)
        .unwrap()
}

fn read_toml_file(file_path: &str) -> io::Result<Value> {
    println!("Reading {}", file_path);
    let contents = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file {}", file_path));
    let parsed_toml: Value = contents
        .parse::<Value>()
        .expect("failed to parse config file");
    Ok(parsed_toml)
}

/// load guest images from config
/// Toml file must be provided to load from memory.
fn load_guest_img(config_toml_path: Option<Vec<OsString>>) -> io::Result<()> {
    let mut f = fs::File::create("./guest.S").unwrap();
    if let Some(config_path) = config_toml_path {
        if let Some(guest_config) = config_path.first() {
            let config =
                read_toml_file(guest_config.to_str().expect("Path contains invalid UTF-8"))
                    .expect("failed to read config file");
            if let Some(image_location) = config.get("image_location") {
                let location: &str = image_location.as_str().unwrap();
                if location == "memory" {
                    let kernel_path = config.get("kernel_path").unwrap();
                    let dtb_incbin = if let Some(dtb_path) = config.get("dtb_path") {
                        format!("    .incbin {}", dtb_path)
                    } else {
                        "".to_string()
                    };
                    writeln!(
                        f,
                        r#"
                    .section .data
                    .global guestkernel_start
                    .global guestkernel_end
                    .align 16
                guestkernel_start:
                    .incbin {}
                guestkernel_end:
                
                  .section .data
                    .global guestdtb_start
                    .global guestdtb_end
                    .align 16
                guestdtb_start:
                    {}
                guestdtb_end:"#,
                        kernel_path, dtb_incbin
                    )?;

                    return Ok(());
                }
            }
        }
    }

    writeln!(f)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let platform = env::var("AX_PLATFORM").unwrap_or("".to_string());
    let platform_family = env::var("AX_PLATFORM").unwrap_or("".to_string());
    let config_toml_path = get_config_paths();
    println!("cargo:rustc-cfg=platform=\"{}\"", platform);
    println!("cargo:rustc-cfg=platform_family=\"{}\"", platform_family);

    let config_files = get_configs();
    let mut output_file = open_output_file();

    println!("cargo:rerun-if-env-changed=UMHV_VM_CONFIGS");
    println!("cargo:rerun-if-changed=build.rs");

    writeln!(
        output_file,
        "pub fn static_vm_configs() -> Vec<&'static str> {{"
    )?;
    match config_files {
        Ok(config_files) => {
            if config_files.is_empty() {
                writeln!(output_file, "    default_static_vm_configs()")?;
            } else {
                writeln!(output_file, "    vec![")?;
                for config_file in config_files {
                    writeln!(output_file, "        r###\"{}\"###,", config_file.content)?;
                    println!(
                        "cargo:rerun-if-changed={}",
                        PathBuf::from(config_file.path).display()
                    );
                }
                writeln!(output_file, "    ]")?;
            }
        }
        Err(error) => {
            writeln!(output_file, "    compile_error!(\"{}\")", error)?;
        }
    }
    writeln!(output_file, "}}")?;

    // load kernel and dtb images
    load_guest_img(config_toml_path)?;

    Ok(())
}
