#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::{Context, Result};
use glob::glob;

use std::cmp::Reverse;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

pub struct Recording(PathBuf);

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
pub struct DirRecordingQueue {
    input_dir: PathBuf,
    queue_dir: PathBuf,
    output_dir: PathBuf,
}

fn create_missing_dir<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    match fs::create_dir_all(dir) {
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        d => d,
    }
}

impl DirRecordingQueue {
    pub fn try_new(input_dir: PathBuf, output_dir: Option<PathBuf>) -> Result<Self> {
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
