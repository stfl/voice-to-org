#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::{Context, Result};
use glob::glob;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use std::cmp::Reverse;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::ErrorKind;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

static RECORDINGS_DIR_IN: &str = "/home/stefan/SmartRecorder";

// static TEST_INPUT_FILE: &str = "/home/stefan/work/cripe/jfk.wav";

struct Recording(PathBuf);

impl From<PathBuf> for Recording {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Deref for Recording {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct DirRecordingQueue {
    input_dir: PathBuf,
    queue_dir: PathBuf,
    output_dir: PathBuf,
}

impl DirRecordingQueue {
    fn try_new(input_dir: PathBuf, output_dir: Option<PathBuf>) -> Result<Self> {
        let input_dir = input_dir.canonicalize()?;

        let queue_dir = input_dir.join("in_process");
        create_missing_dir(&queue_dir)?;
        let output_dir = output_dir.unwrap_or(input_dir.join("processed"));
        create_missing_dir(&output_dir)?;

        let queue = Self {
            input_dir,
            queue_dir,
            output_dir,
        };
        queue.empty_processing_queue()?;

        Ok(queue)
    }

    fn find_latest_new_recording(&self) -> Result<Option<PathBuf>> {
        let mut paths = fs::read_dir(&self.input_dir)?
            .filter_map(Result::ok)
            .map(|de| de.path())
            .filter(|p| match p.extension() {
                Some(ex) if ex == "wav" => true,
                _ => false,
            })
            .collect::<Vec<_>>();

        paths.sort_unstable_by_key(|p| {
            let metadata = fs::metadata(p).unwrap();
            Reverse(metadata.modified().unwrap())
        });

        Ok(paths.into_iter().nth(0))
    }

    fn empty_processing_queue(&self) -> Result<()> {
        println!("emptying processing queue");
        let pattern = &format!(
            "{queue_dir}/*.wav",
            queue_dir = self
                .queue_dir
                .as_path()
                .to_str()
                .context("cannot convert path to str")?
        );
        for file in glob(pattern)
            .context(format!(
                "failed reading files in processing queue in {queue:?}",
                queue = self.queue_dir
            ))?
            .filter_map(Result::ok)
        {
            let dest = self
                .input_dir
                .join(file.file_name().context("no file_name found")?);
            fs::rename(file, &dest)?;

            debug_assert!(dest.is_file());
            println!(
                "moved {f:?} out of the processing queue",
                f = dest.file_name().unwrap()
            );
        }
        Ok(())
    }

    fn move_file_to_processing_queue(&self, file: PathBuf) -> Result<PathBuf> {
        let dest = self
            .queue_dir
            .join(file.file_name().context("no file_name found")?);
        fs::rename(file, &dest)?;
        debug_assert!(dest.is_file());
        Ok(dest)
    }

    fn move_file_to_out_dir(&self, file: PathBuf) -> Result<PathBuf> {
        let dest = self
            .output_dir
            .join(file.file_name().context("no file_name found")?);
        fs::rename(file, &dest)?;
        debug_assert!(dest.is_file());
        Ok(dest)
    }
}

impl Drop for DirRecordingQueue {
    fn drop(&mut self) {
        println!("Drop");
        self.empty_processing_queue()
            .expect("failed emptying the processing queue");
    }
}

impl Iterator for DirRecordingQueue {
    type Item = Recording;

    fn next(&mut self) -> Option<Self::Item> {
        self.find_latest_new_recording()
            .expect("failed finding latest recording")
            .map(|rec| {
                self.move_file_to_processing_queue(rec)
                    .expect("failed moving the to processing queue")
                    .into()
            })
    }
}

fn transcribe_audio(input_file_path: &Path, model: &str, lang: Option<&str>) -> Result<String> {
    let output_dir_str = "/tmp";
    let output_dir = Path::new(output_dir_str);
    let output_path = output_dir.join(
        input_file_path
            .file_name()
            .map(|f| Path::new(f).with_extension("json"))
            .context("cannot get file_name from {path}")?,
    );

    let mut cmd = Command::new("whisper"); // whisper needs to be in PATH
    cmd.arg(input_file_path)
        .args(["--output_format", "json"])
        .args(["--output_dir", output_dir_str])
        .args(["--model", model])
        .stdout(Stdio::inherit());

    if let Some(l) = lang {
        cmd.args(["--language", l]);
    }

    cmd.output()?;

    debug_assert!(output_path.is_file());
    let output_file = File::open(&output_path)?;

    #[derive(Deserialize, Debug)]
    struct WhisperOutput {
        text: String,
    }

    let out: WhisperOutput = serde_json::from_reader(BufReader::new(output_file))?;

    Ok(out.text.trim().into())
}

fn create_missing_dir<P: AsRef<Path>>(dir: P) -> std::io::Result<()> {
    match fs::create_dir_all(dir) {
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        d => d,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    debug_assert!(Path::new(RECORDINGS_DIR_IN).is_dir());

    let queue = DirRecordingQueue::try_new(RECORDINGS_DIR_IN.into(), None)?;

    println!("{queue:?}");

    for rec in queue {
        println!("{rec:?}", rec = *rec);
    }

    // while let Ok(Some(rec)) = find_latest_new_recording(RECORDINGS_DIR_IN) {
    //     println!("transcribing recording {rec:?}");
    //     let tmp_path = move_file_to_processing_queue(rec)?;
    //     let text = transcribe_audio(&tmp_path, "large", None)?;
    //     move_file_to_out_dir(tmp_path)?;
    //     println!("Text: {text}")
    // }

    Ok(())
}
