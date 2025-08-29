use std::{
    error::Error,
    fs::{File, Metadata},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

enum BufferMessage {
    Chunk(Vec<u8>),
    Truncated,
}
enum LinesMessage {
    Lines(Vec<String>),
    Truncated,
}

pub struct Buffer {
    content: Vec<String>,
    lines_receiver: Receiver<LinesMessage>,
}

impl Buffer {
    pub fn new(file_path: Option<PathBuf>) -> Result<Buffer, Box<dyn Error>> {
        let (lines_sender, lines_receiver) = mpsc::channel();

        if let Some(path) = file_path {
            let (bytes_sender, bytes_receiver) = mpsc::channel();

            thread::spawn(move || {
                if let Ok(mut file) = File::open(&path) {
                    let mut buffer = vec![0; 64 * 1024];
                    let mut last_metadata: Option<Metadata> = None;

                    loop {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            match &last_metadata {
                                Some(prev_meta) => {
                                    // File was truncated (size decreased)
                                    if metadata.len() < prev_meta.len() {
                                        if bytes_sender.send(BufferMessage::Truncated).is_err() {
                                            break;
                                        }
                                        let _ = file.seek(SeekFrom::Start(0));
                                    }
                                    // File was rotated (different file)
                                    else if metadata.modified().ok() != prev_meta.modified().ok()
                                    {
                                        if let Ok(new_file) = File::open(&path) {
                                            file = new_file;
                                        }
                                    }
                                }
                                None => {}
                            }
                            last_metadata = Some(metadata);
                        }
                        match file.read(&mut buffer) {
                            Ok(0) => {
                                thread::sleep(Duration::from_millis(100));
                            }
                            Ok(bytes_read) => {
                                let chunk = buffer[..bytes_read].to_vec();
                                if bytes_sender.send(BufferMessage::Chunk(chunk)).is_err() {
                                    break;
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            });

            thread::spawn(move || {
                let mut leftover = Vec::new();
                let mut line_buffer = Vec::new();

                while let Ok(buffer_message) = bytes_receiver.recv() {
                    match buffer_message {
                        BufferMessage::Truncated => {
                            if lines_sender.send(LinesMessage::Truncated).is_err() {
                                return;
                            }
                        }
                        BufferMessage::Chunk(chunk) => {
                            leftover.extend(chunk);

                            let mut last_newline = 0;
                            for (i, &byte) in leftover.iter().enumerate() {
                                if byte == b'\n' {
                                    if let Ok(line) =
                                        String::from_utf8(leftover[last_newline..i].to_vec())
                                    {
                                        line_buffer.push(line.trim_end_matches('\r').to_string());
                                    }
                                    last_newline = i + 1;
                                }
                            }

                            if lines_sender
                                .send(LinesMessage::Lines(line_buffer.clone()))
                                .is_err()
                            {
                                return;
                            }
                            line_buffer.clear();

                            leftover = leftover[last_newline..].to_vec();
                        }
                    }
                }

                // Handle final incomplete line and remaining buffered lines
                if !leftover.is_empty() {
                    if let Ok(line) = String::from_utf8(leftover) {
                        line_buffer.push(line.trim_end_matches('\r').to_string());
                    }
                }

                if !line_buffer.is_empty() {
                    let _ = lines_sender.send(LinesMessage::Lines(line_buffer));
                }
            });
        }

        Ok(Buffer {
            content: Vec::new(),
            lines_receiver: lines_receiver,
        })
    }

    pub fn get_content(&mut self) -> &Vec<String> {
        while let Ok(message) = self.lines_receiver.try_recv() {
            match message {
                LinesMessage::Lines(lines) => {
                    self.content.extend(lines);
                }
                LinesMessage::Truncated => {
                    self.content.clear();
                }
            }
        }
        &self.content
    }
}
