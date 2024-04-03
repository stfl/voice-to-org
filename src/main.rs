use anyhow::Result;
use std::path::Path;

static RECORDINGS_DIR_IN: &str = "/home/stefan/SmartRecorder";
static WHISPER_MODEL: &str = "base";

mod dir_queue;
mod interpret;
mod transcribe;

use dir_queue::DirRecordingQueue;
use transcribe::Transcribe;

// static TEST_INPUT_FILE: &str = "/home/stefan/work/cripe/jfk.wav";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    debug_assert!(Path::new(RECORDINGS_DIR_IN).is_dir());

    let queue = DirRecordingQueue::try_new(RECORDINGS_DIR_IN.into(), None)?;

    println!("{queue:?}");

    for rec in queue {
        let transcription = Transcribe::new(rec.to_owned(), WHISPER_MODEL.into()).transcribe()?;
        println!("{transcription}");
    }

    Ok(())
}
