use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self};
use std::path::{Path, PathBuf};

use clap::Parser;
use zip::read::ZipArchive;

/// Simple program to extract files of a specific type from zip files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to a zip file or a directory containing zip files.
    #[arg(short, long, value_name = "INPUT")]
    input: PathBuf,

    /// File extension to filter for (e.g., "txt" or "png"). You may omit the dot.
    #[arg(short, long, value_name = "EXTENSION")]
    extension: String,

    /// Output directory where the extracted files will be saved.
    #[arg(short, long, value_name = "OUTPUT")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments.
    let args = Args::parse();

    // Ensure the output directory exists.
    fs::create_dir_all(&args.output)?;

    // Prepare the extension filter in lower-case, without a leading dot.
    let filter_ext = args.extension.trim_start_matches('.').to_lowercase();

    // Determine if the input path is a file or a directory.
    if args.input.is_dir() {
        // Process all .zip files in the given directory (non-recursive).
        for entry in fs::read_dir(&args.input)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(|s| s.eq_ignore_ascii_case("zip"))
                    .unwrap_or(false)
            {
                println!("Processing zip file: {}", path.display());
                if let Err(e) = process_zip_file(&path, &filter_ext, &args.output) {
                    eprintln!("Error processing {}: {}", path.display(), e);
                }
            }
        }
    } else if args.input.is_file() {
        // Process a single zip file.
        println!("Processing zip file: {}", args.input.display());
        process_zip_file(&args.input, &filter_ext, &args.output)?;
    } else {
        return Err(format!("Input path {} is not a valid file or directory.", args.input.display()).into());
    }

    Ok(())
}

/// Processes a single zip file by extracting all files that match the given extension.
/// Files whose names include "__MACOSX" are skipped.
/// The extracted files are saved to `output_dir` using their original file names.
/// Note: if multiple files share the same name, later files will overwrite earlier ones.
fn process_zip_file(zip_path: &Path, ext: &str, output_dir: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        let entry_name = zip_file.name();

        // Skip any entry that is part of the "__MACOSX" metadata.
        if entry_name.contains("__MACOSX") {
            continue;
        }

        // Only process file entries (skip directories).
        if zip_file.is_file() {
            let entry_path = Path::new(entry_name);

            // Check if the file's extension matches the desired filter.
            if let Some(entry_ext) = entry_path.extension().and_then(OsStr::to_str) {
                if entry_ext.to_lowercase() == ext {
                    // Get the original file name (the last component of the path).
                    if let Some(file_name) = entry_path.file_name() {
                        let output_file_path = output_dir.join(file_name);

                        // Create and write the output file.
                        let mut outfile = File::create(&output_file_path)?;
                        io::copy(&mut zip_file, &mut outfile)?;
                        println!("Extracted: {}", output_file_path.display());
                    } else {
                        eprintln!("Warning: Skipping an entry with no valid file name: {}", entry_name);
                    }
                }
            }
        }
    }

    Ok(())
}
