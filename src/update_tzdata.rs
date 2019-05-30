extern crate tz_lookup_simple;

use bincode::serialize_into;
use brotli::CompressorWriter;
use progress_streams::{ProgressReader, ProgressWriter};
use reqwest::get;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use termion::clear;
use tz_lookup_simple::TzLookup;
use zip::read::ZipArchive;

fn read_with_progress(reader: impl Read, msg: &'static str) -> (Arc<AtomicBool>, impl Read) {
    let total = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let res = ProgressReader::new(reader, {
        let total = total.clone();
        move |progress| {
            total.fetch_add(progress, Ordering::SeqCst);
        }
    });
    {
        let total = total.clone();
        let done = done.clone();
        thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                print!(
                    "\r{}: {} KiB{}",
                    msg,
                    total.load(Ordering::SeqCst) / 1024,
                    clear::AfterCursor
                );
                thread::sleep(Duration::from_millis(16));
            }
        });
    }
    (done, res)
}

fn write_with_progress(writer: impl Write, msg: &'static str) -> (Arc<AtomicBool>, impl Write) {
    let total = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let res = ProgressWriter::new(writer, {
        let total = total.clone();
        move |progress| {
            total.fetch_add(progress, Ordering::SeqCst);
        }
    });
    {
        let total = total.clone();
        let done = done.clone();
        thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                print!(
                    "\r{}: {} KiB{}",
                    msg,
                    total.load(Ordering::SeqCst) / 1024,
                    clear::AfterCursor
                );
                thread::sleep(Duration::from_millis(16));
            }
        });
    }
    (done, res)
}

fn main() {
    const TIMEZONE_GEOJSON_URL: &'static str = "https://github.com/evansiroky/timezone-boundary-builder/releases/download/2019a/timezones-with-oceans.geojson.zip";
    const GEOJSON_PATH: &'static str = "dist/combined-with-oceans.json";
    println!("starting request...");
    let req = get(TIMEZONE_GEOJSON_URL).unwrap();

    let (done, mut req_reader) = read_with_progress(req, "reading geojson");
    let mut json_dat = Vec::with_capacity(50000000);

    req_reader.read_to_end(&mut json_dat).unwrap();
    done.store(true, Ordering::SeqCst);
    println!("\rdone downloading json{}", clear::AfterCursor);

    let json_dat_cursor = Cursor::new(json_dat);

    let mut archive = ZipArchive::new(json_dat_cursor).unwrap();
    let geo_json = archive.by_name(GEOJSON_PATH).unwrap();

    let (done, parse_read) = read_with_progress(geo_json, "parsing json");

    let tzl = TzLookup::new(parse_read).expect("error building TzLookup from geojson data");

    done.store(true, Ordering::SeqCst);
    println!("\rdone parsing json{}", clear::AfterCursor);

    let out = File::create("./src/tzdata_complete.bin.brotli").unwrap();
    let out = CompressorWriter::new(out, 4096, 11, 21);

    let (done, out_writer) = write_with_progress(out, "writing output");

    serialize_into(out_writer, &tzl).unwrap();

    done.store(true, Ordering::SeqCst);
    println!("\rdone writing output{}", clear::AfterCursor);

    println!("tzdata has been updated!");
}
