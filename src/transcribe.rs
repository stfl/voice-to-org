#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::Deserialize;

use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

pub struct Transcribe {
    audio_file: PathBuf,
    model: String,
    lang: Option<String>,
    temperature: Option<f64>,
}

fn remove_file_if_exists<P: AsRef<Path>>(file: P) -> io::Result<()> {
    match fs::remove_file(&file) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        d => d,
    }
}

impl Transcribe {
    pub fn new(audio_file: PathBuf, model: String) -> Self {
        Self {
            audio_file,
            model,
            lang: None,
            temperature: None,
        }
    }

    pub fn lang(mut self, lang: String) -> Self {
        self.lang = Some(lang);
        self
    }

    pub fn temperature(mut self, temp: f64) -> Self {
        assert!(
            temp >= 0. && temp <= 2.,
            "Temperature needs to be between 0 and 2"
        );
        self.temperature = Some(temp);
        self
    }

    pub fn transcribe(&self) -> Result<String> {
        let res_dir = Path::new("/tmp");
        let res_file_name = self
            .audio_file
            .file_name()
            .map(|f| Path::new(f).with_extension("json"))
            .context("cannot get file_name from {path}")?;

        let res_path = res_dir.join(res_file_name);
        remove_file_if_exists(&res_path)?;

        let mut cmd = Command::new("whisper"); // whisper needs to be in PATH
        cmd.arg(&self.audio_file)
            .args(["--output_format", "json"])
            .args(["--output_dir", res_dir.to_str().unwrap()])
            .args(["--model", &self.model])
            .stdout(Stdio::inherit());

        if let Some(ref val) = self.lang {
            cmd.args(["--language", val]);
        }

        if let Some(ref val) = self.temperature {
            cmd.args(["--temperature", &format!("{val:.2}")]);
        }

        cmd.output()?;

        let output_file =
            File::open(&res_path).context("whisper did not producse the expected output file")?;

        #[derive(Deserialize, Debug)]
        struct WhisperOutput {
            text: String,
        }

        let out: WhisperOutput = serde_json::from_reader(BufReader::new(output_file))?;

        Ok(out.text.trim().into())
    }
}
