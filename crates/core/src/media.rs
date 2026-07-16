use std::path::Path;
use std::process::Command;

use rusqlite::Connection;
use serde::Serialize;

use crate::Result;

#[derive(Debug, Clone, Serialize)]
pub struct MediaInfo {
    pub duration_ms: i64,
    pub width: i64,
    pub height: i64,
    pub fps: Option<f64>,
    pub codec: Option<String>,
}

/// Probe a media file with ffprobe. Returns None when ffprobe is not
/// installed or the file can't be parsed — callers must degrade gracefully.
pub fn probe(path: &Path) -> Option<MediaInfo> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-print_format", "json",
            "-show_format", "-show_streams",
        ])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let duration_s: f64 = v["format"]["duration"].as_str()?.parse().ok()?;
    let video = v["streams"]
        .as_array()?
        .iter()
        .find(|s| s["codec_type"] == "video")?;
    let fps = video["r_frame_rate"].as_str().and_then(|r| {
        let (num, den) = r.split_once('/')?;
        let (num, den): (f64, f64) = (num.parse().ok()?, den.parse().ok()?);
        if den > 0.0 { Some(num / den) } else { None }
    });
    Some(MediaInfo {
        duration_ms: (duration_s * 1000.0) as i64,
        width: video["width"].as_i64()?,
        height: video["height"].as_i64()?,
        fps,
        codec: video["codec_name"].as_str().map(str::to_string),
    })
}

/// Probe and persist metadata onto an asset row (best-effort).
pub fn enrich_asset(conn: &Connection, asset_id: &str, path: &Path) -> Result<Option<MediaInfo>> {
    let Some(info) = probe(path) else { return Ok(None) };
    conn.execute(
        "UPDATE assets SET duration_ms = ?1, width = ?2, height = ?3, fps = ?4, codec = ?5
         WHERE id = ?6",
        rusqlite::params![info.duration_ms, info.width, info.height, info.fps, info.codec, asset_id],
    )?;
    Ok(Some(info))
}
