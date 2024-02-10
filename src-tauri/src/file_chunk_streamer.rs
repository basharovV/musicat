// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod FileChunkHandler {

    use http_range::HttpRange;
    use std::sync::{Arc, Mutex};
    use std::{
        io::{Read, Seek, SeekFrom, Write},
        path::PathBuf,
        process::{Command, Stdio},
    };
    use tauri::http::{header::*, status::StatusCode, ResponseBuilder};
    use tauri_utils::mime_type::MimeType;

    /**
     * Grab the PCM samples to send to the webview. 
     */
    pub fn get_stream_response(
        request: &tauri::http::Request,
        boundary_id: &Arc<Mutex<i32>>,
    ) -> Result<tauri::http::Response, Box<dyn std::error::Error>> {
        println!("File chunk handler");
        // skip leading `/`
        let path = percent_encoding::percent_decode(request.uri().as_bytes())
            .decode_utf8_lossy()
            .to_string()
            .replace("stream://localhost/", "");

        println!("File chunk handler:path {:?}", &path);
        let mut file = std::fs::File::open(&path)?;
        // get file length
        let len = {
            let old_pos = file.stream_position()?;
            let len = file.seek(SeekFrom::End(0))?;
            file.seek(SeekFrom::Start(old_pos))?;
            len
        };

        println!("File chunk handler: Length: {:?}", len);
        // get file mime type
        let (mime_type, read_bytes) = {
            let nbytes = len.min(8192);
            let mut magic_buf = Vec::with_capacity(nbytes as usize);
            let old_pos = file.stream_position()?;
            (&mut file).take(nbytes).read_to_end(&mut magic_buf)?;
            file.seek(SeekFrom::Start(old_pos))?;
            (
                MimeType::parse(&magic_buf, &path),
                // return the `magic_bytes` if we read the whole file
                // to avoid reading it again later if this is not a range request
                if len < 8192 { Some(magic_buf) } else { None },
            )
        };
        println!("File chunk handler: MimeType: {:?}", mime_type);

        let mut resp = ResponseBuilder::new().header(CONTENT_TYPE, &mime_type);

        // if the webview sent a range header, we need to send a 206 in return
        // Actually only macOS and Windows are supported. Linux will ALWAYS return empty headers.
        let http_response = if let Some(range_header) = request.headers().get("range") {
            let not_satisfiable = || {
                ResponseBuilder::new()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(CONTENT_RANGE, format!("bytes */{len}"))
                    .body(vec![])
            };

            // parse range header
            let ranges = if let Ok(ranges) = HttpRange::parse(range_header.to_str()?, len) {
                ranges
                    .iter()
                    // map the output back to spec range <start-end>, example: 0-499
                    .map(|r| (r.start, r.start + r.length - 1))
                    .collect::<Vec<_>>()
            } else {
                return Ok(not_satisfiable()?);
            };

            /// The Maximum bytes we send in one range
            const MAX_LEN: u64 = 1000 * 1024;
            println!("File chunk handler: ranges: {:?}", ranges);

            if ranges.len() == 1 {
                let &(start, mut end) = ranges.first().unwrap();

                // check if a range is not satisfiable
                //
                // this should be already taken care of by HttpRange::parse
                // but checking here again for extra assurance
                if start >= len || end >= len || end < start {
                    return Ok(not_satisfiable()?);
                }

                // adjust end byte for MAX_LEN
                end = start + (end - start).min(len - start).min(MAX_LEN - 1);

                // calculate number of bytes needed to be read
                let bytes_to_read = end + 1 - start;
                println!("File chunk handler: bytes: {:?}", bytes_to_read);

                // allocate a buf with a suitable capacity
                let mut buf = Vec::with_capacity(bytes_to_read as usize);
                // seek the file to the starting byte
                file.seek(SeekFrom::Start(start))?;
                // read the needed bytes
                file.take(bytes_to_read).read_to_end(&mut buf)?;
                println!("File chunk handler: read: {:?}", bytes_to_read);

                resp = resp.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
                resp = resp.header(CONTENT_LENGTH, end + 1 - start);
                resp = resp.header(ACCESS_CONTROL_ALLOW_ORIGIN,"*");
                resp = resp.status(StatusCode::PARTIAL_CONTENT);
                println!("File chunk handler: resp: {:?}", resp);

                resp.body(buf)
            } else {
                let mut buf = Vec::new();
                let ranges = ranges
                    .iter()
                    .filter_map(|&(start, mut end)| {
                        // filter out unsatisfiable ranges
                        //
                        // this should be already taken care of by HttpRange::parse
                        // but checking here again for extra assurance
                        if start >= len || end >= len || end < start {
                            None
                        } else {
                            // adjust end byte for MAX_LEN
                            end = start + (end - start).min(len - start).min(MAX_LEN - 1);
                            Some((start, end))
                        }
                    })
                    .collect::<Vec<_>>();

                let mut id = boundary_id.lock().unwrap();
                *id += 1;
                let boundary = format!("sadasq2e{id}");
                let boundary_sep = format!("\r\n--{boundary}\r\n");
                let boundary_closer = format!("\r\n--{boundary}\r\n");

                resp = resp.header(
                    CONTENT_TYPE,
                    format!("multipart/byteranges; boundary={boundary}"),
                );

                for (end, start) in ranges {
                    // a new range is being written, write the range boundary
                    buf.write_all(boundary_sep.as_bytes())?;

                    // write the needed headers `Content-Type` and `Content-Range`
                    buf.write_all(format!("{CONTENT_TYPE}: {mime_type}\r\n").as_bytes())?;
                    buf.write_all(
                        format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes(),
                    )?;

                    // write the separator to indicate the start of the range body
                    buf.write_all("\r\n".as_bytes())?;

                    // calculate number of bytes needed to be read
                    let bytes_to_read = end + 1 - start;
                    
                    let mut local_buf = vec![0_u8; bytes_to_read as usize];
                    file.seek(SeekFrom::Start(start))?;
                    file.read_exact(&mut local_buf)?;
                    buf.extend_from_slice(&local_buf);
                }
                // all ranges have been written, write the closing boundary
                buf.write_all(boundary_closer.as_bytes())?;

                resp.body(buf)
            }
        } else {
            resp = resp.header(CONTENT_LENGTH, len);
            let mut buf = Vec::with_capacity(len as usize);
            file.read_to_end(&mut buf)?;
            resp.body(buf)
        };

        http_response.map_err(Into::into)
    }
}
