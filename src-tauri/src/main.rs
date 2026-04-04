#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::Serialize;

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn sanitize_filename(raw: &str) -> String {
    let mut s = raw
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string();
    if s.is_empty() {
        s = "无题".to_string();
    }
    if s.len() > 40 {
        s.truncate(40);
    }
    s
}

fn sanitize_audio_ext(raw: &str) -> String {
    let filtered: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    if filtered.is_empty() {
        "mp3".to_string()
    } else {
        filtered
    }
}

fn get_ffmpeg_cmd() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    if let Ok(custom) = std::env::var("FFMPEG_PATH") {
        let p = PathBuf::from(custom);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let near = dir.join(exe_name);
            if near.exists() {
                return Some(near);
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let near = cwd.join(exe_name);
        if near.exists() {
            return Some(near);
        }
        let in_release = cwd.join("src-tauri").join("target").join("release").join(exe_name);
        if in_release.exists() {
            return Some(in_release);
        }
    }
    Some(PathBuf::from(exe_name))
}

fn get_ffprobe_cmd() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };
    if let Ok(custom) = std::env::var("FFPROBE_PATH") {
        let p = PathBuf::from(custom);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(ffmpeg) = std::env::var("FFMPEG_PATH") {
        let p = PathBuf::from(ffmpeg);
        if let Some(parent) = p.parent() {
            let near = parent.join(exe_name);
            if near.exists() {
                return Some(near);
            }
        }
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let near = dir.join(exe_name);
            if near.exists() {
                return Some(near);
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let near = cwd.join(exe_name);
        if near.exists() {
            return Some(near);
        }
        let in_release = cwd.join("src-tauri").join("target").join("release").join(exe_name);
        if in_release.exists() {
            return Some(in_release);
        }
    }
    Some(PathBuf::from(exe_name))
}

#[derive(Serialize)]
struct AudioTagInfo {
    title: String,
    artist: String,
    album: String,
    duration_sec: f64,
    cover_data_url: String,
}

fn parse_time_secs(raw: &str) -> Option<f64> {
    let parts: Vec<&str> = raw.trim().split(':').collect();
    if parts.is_empty() {
        return None;
    }
    if parts.len() == 1 {
        return parts[0].parse::<f64>().ok();
    }
    if parts.len() == 2 {
        let m = parts[0].parse::<f64>().ok()?;
        let s = parts[1].parse::<f64>().ok()?;
        return Some(m * 60.0 + s);
    }
    if parts.len() == 3 {
        let h = parts[0].parse::<f64>().ok()?;
        let m = parts[1].parse::<f64>().ok()?;
        let s = parts[2].parse::<f64>().ok()?;
        return Some(h * 3600.0 + m * 60.0 + s);
    }
    None
}

fn compute_clip_args(start_sec: String, end_sec: String) -> Result<Vec<String>, String> {
    let start = parse_time_secs(&start_sec).unwrap_or(0.0).max(0.0);
    let end = parse_time_secs(&end_sec).unwrap_or(0.0).max(0.0);
    if end > 0.0 && end <= start {
        return Err("结束时间必须大于开始时间".to_string());
    }
    let mut args = Vec::new();
    if start > 0.0 {
        args.push("-ss".to_string());
        args.push(format!("{start:.3}"));
    }
    if end > 0.0 {
        args.push("-t".to_string());
        args.push(format!("{:.3}", end - start));
    } else {
        args.push("-shortest".to_string());
    }
    Ok(args)
}

#[tauri::command]
fn inspect_audio_tags(audio_base64: String, audio_extension: String) -> Result<AudioTagInfo, String> {
    let audio_bytes = base64::engine::general_purpose::STANDARD
        .decode(audio_base64)
        .map_err(|_| "音频解码失败".to_string())?;
    let work_dir = std::env::temp_dir().join(format!("poem_audio_meta_{}", now_millis()));
    fs::create_dir_all(&work_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;
    let audio_ext = sanitize_audio_ext(&audio_extension);
    let audio_path = work_dir.join(format!("meta.{audio_ext}"));
    fs::write(&audio_path, audio_bytes).map_err(|e| format!("写入音频失败: {e}"))?;

    let ffprobe = get_ffprobe_cmd().ok_or_else(|| "未找到 ffprobe".to_string())?;
    let probe = Command::new(ffprobe)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format_tags=title,artist,album:format=duration")
        .arg("-of")
        .arg("default=nw=1:nk=0")
        .arg(&audio_path)
        .output()
        .map_err(|e| format!("读取音频标签失败: {e}"))?;

    let text = String::from_utf8_lossy(&probe.stdout).to_string();
    let mut title = String::new();
    let mut artist = String::new();
    let mut album = String::new();
    let mut duration_sec = 0.0f64;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("TAG:title=") {
            title = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("TAG:artist=") {
            artist = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("TAG:album=") {
            album = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("duration=") {
            duration_sec = v.trim().parse::<f64>().unwrap_or(0.0);
        }
    }

    let mut cover_data_url = String::new();
    let ffmpeg = get_ffmpeg_cmd().ok_or_else(|| "未找到 FFmpeg".to_string())?;
    let cover_path = work_dir.join("cover.jpg");
    let cover_result = Command::new(ffmpeg)
        .arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(&audio_path)
        .arg("-map")
        .arg("0:v")
        .arg("-frames:v")
        .arg("1")
        .arg(&cover_path)
        .output();
    if cover_result.is_ok() && cover_path.exists() {
        if let Ok(bytes) = fs::read(&cover_path) {
            let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
            cover_data_url = format!("data:image/jpeg;base64,{b64}");
        }
    }

    let _ = fs::remove_dir_all(&work_dir);
    Ok(AudioTagInfo {
        title,
        artist,
        album,
        duration_sec,
        cover_data_url,
    })
}

#[tauri::command]
fn export_video_ffmpeg(
    image_data_urls: Vec<String>,
    audio_base64: String,
    audio_extension: String,
    title: String,
    start_sec: String,
    end_sec: String,
) -> Result<String, String> {
    if image_data_urls.is_empty() {
        return Err("没有图片帧数据".to_string());
    }

    let audio_bytes = base64::engine::general_purpose::STANDARD
        .decode(&audio_base64)
        .map_err(|_| "音频解码失败".to_string())?;

    let work_dir = std::env::temp_dir().join(format!("poem_video_{}", now_millis()));
    fs::create_dir_all(&work_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;

    let audio_ext = sanitize_audio_ext(&audio_extension);
    let audio_path = work_dir.join(format!("audio.{audio_ext}"));
    fs::write(&audio_path, &audio_bytes).map_err(|e| format!("写入音频失败: {e}"))?;

    // Write each frame as a separate PNG file
    for (i, data_url) in image_data_urls.iter().enumerate() {
        let image_b64 = data_url
            .split_once(',')
            .map(|(_, b64)| b64.to_string())
            .ok_or_else(|| "图片数据无效".to_string())?;
        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(&image_b64)
            .map_err(|_| "图片解码失败".to_string())?;
        let image_path = work_dir.join(format!("frame{:03}.png", i));
        fs::write(&image_path, &image_bytes).map_err(|e| format!("写入图片失败: {e}"))?;
    }

    let out_dir = dirs::download_dir()
        .or_else(|| dirs::desktop_dir())
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| "无法确定导出目录".to_string())?;
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).map_err(|e| format!("创建导出目录失败: {e}"))?;
    }
    let out_name = format!("古诗视频_{}_{}.mp4", sanitize_filename(&title), now_millis());
    let out_path = out_dir.join(out_name);

    let ffmpeg = get_ffmpeg_cmd().ok_or_else(|| "未找到 FFmpeg".to_string())?;
    let build_args = |audio_codec: &str| -> Result<Vec<String>, String> {
        let mut args = vec![
            "-y".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-framerate".to_string(),
            "1".to_string(),
            "-i".to_string(),
            work_dir.join("frame%03d.png").to_string_lossy().to_string(),
            "-i".to_string(),
            audio_path.to_string_lossy().to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryslow".to_string(),
            "-crf".to_string(),
            "6".to_string(),
            "-vf".to_string(),
            "scale=trunc(iw/2)*2:trunc(ih/2)*2,format=yuv420p".to_string(),
            "-c:a".to_string(),
            audio_codec.to_string(),
            "-b:a".to_string(),
            "320k".to_string(),
            "-movflags".to_string(),
            "+faststart".to_string(),
        ];
        let clip_args = compute_clip_args(start_sec.clone(), end_sec.clone())?;
        args.extend(clip_args);
        args.push(out_path.to_string_lossy().to_string());
        Ok(args)
    };

    let mut last_error = String::new();
    for codec in ["aac", "libmp3lame"] {
        let args = build_args(codec)?;
        let output = Command::new(&ffmpeg)
            .args(&args)
            .output()
            .map_err(|e| format!("FFmpeg 执行失败，请确认已安装并加入 PATH: {e}"))?;
        if output.status.success() {
            let _ = fs::remove_dir_all(&work_dir);
            return Ok(out_path.to_string_lossy().to_string());
        }
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
        last_error = format!(
            "codec={codec}, code={:?}, stderr={}, stdout={}",
            output.status.code(),
            if err.is_empty() { "<empty>" } else { &err },
            if out.is_empty() { "<empty>" } else { &out }
        );
    }

    let _ = fs::remove_dir_all(&work_dir);
    Err(format!("FFmpeg 合成失败: {last_error}"))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![export_video_ffmpeg, inspect_audio_tags])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
