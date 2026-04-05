#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use lofty::prelude::{ItemKey, *};
use lofty::file::AudioFile;
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

fn _get_ffprobe_cmd() -> Option<PathBuf> {
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
#[serde(rename_all = "camelCase")]
struct AudioTagInfo {
    title: String,
    artist: String,
    album: String,
    duration_sec: f64,
    cover_data_url: String,
    lyrics: String,
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

#[tauri::command]
fn inspect_audio_tags(audio_base64: String, audio_extension: String) -> Result<AudioTagInfo, String> {
    let audio_bytes = base64::engine::general_purpose::STANDARD
        .decode(&audio_base64)
        .map_err(|_| "音频解码失败".to_string())?;

    let work_dir = std::env::temp_dir().join(format!("poem_audio_meta_{}", now_millis()));
    fs::create_dir_all(&work_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
    let audio_ext = sanitize_audio_ext(&audio_extension);
    let audio_path = work_dir.join(format!("meta.{}", audio_ext));
    fs::write(&audio_path, &audio_bytes).map_err(|e| format!("写入音频失败: {}", e))?;

    let mut file = std::fs::File::open(&audio_path).map_err(|e| format!("打开音频文件失败: {}", e))?;
    let tagged_file = lofty::read_from(&mut file)
        .map_err(|e| format!("读取音频标签失败: {}", e))?;

    let _ = fs::remove_dir_all(&work_dir);

    let properties = tagged_file.properties();
    let duration_sec = properties.duration().as_secs_f64();

    let mut title = String::new();
    let mut artist = String::new();
    let mut album = String::new();
    let mut cover_data_url = String::new();
    let mut lyrics = String::new();

    // 遍历所有标签 (ID3v2, ID3v1, Vorbis, etc.)
    for tag in tagged_file.tags() {
        // 基本信息
        if title.is_empty() {
            title = tag.title().map(|s| s.to_string()).unwrap_or_default();
        }
        if artist.is_empty() {
            artist = tag.artist().map(|s| s.to_string()).unwrap_or_default();
        }
        if album.is_empty() {
            album = tag.album().map(|s| s.to_string()).unwrap_or_default();
        }

        // 收集所有图片 (遍历每个 tag 的所有图片)
        for picture in tag.pictures() {
            if cover_data_url.is_empty() {
                let mime_str = picture.mime_type()
                    .map(|m| m.as_str())
                    .unwrap_or("image/jpeg");
                let base64_data = base64::engine::general_purpose::STANDARD.encode(picture.data());
                cover_data_url = format!("data:{};base64,{}", mime_str, base64_data);
            }
        }

        // 尝试读取歌词
        if lyrics.is_empty() {
            if let Some(val) = tag.get_string(&ItemKey::Lyrics) {
                lyrics = val.to_string();
            }
        }
        if lyrics.is_empty() {
            if let Some(val) = tag.get_string(&ItemKey::Comment) {
                lyrics = val.to_string();
            }
        }
    }

    Ok(AudioTagInfo {
        title,
        artist,
        album,
        duration_sec,
        cover_data_url,
        lyrics,
    })
}

#[tauri::command]
fn export_video_ffmpeg(
    _poem_card_png: String,
    _poem_card_w: u32,
    _poem_card_h: u32,
    _media_area_y: i32,
    _media_area_h: i32,
    media_items_json: String,
    audio_base64: String,
    audio_extension: String,
    title: String,
    start_sec: String,
    end_sec: String,
    _total_duration: f64,
    frame_data_urls: Vec<String>,
    fps: u32,
) -> Result<String, String> {
    let _media_items: Vec<serde_json::Value> = serde_json::from_str(&media_items_json)
        .map_err(|e| format!("媒体数据解析失败: {e}"))?;

    let work_dir = std::env::temp_dir().join(format!("poem_video_{}", now_millis()));
    fs::create_dir_all(&work_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;

    let has_audio = !audio_base64.is_empty();
    let audio_path: Option<PathBuf> = if has_audio {
        let audio_bytes = base64::engine::general_purpose::STANDARD
            .decode(&audio_base64)
            .map_err(|_| "音频解码失败".to_string())?;
        let audio_ext = sanitize_audio_ext(&audio_extension);
        let path = work_dir.join(format!("audio.{audio_ext}"));
        fs::write(&path, &audio_bytes).map_err(|e| format!("写入音频失败: {e}"))?;
        Some(path)
    } else {
        None
    };

    let final_path = work_dir.join("output.mp4");

    if frame_data_urls.is_empty() {
        return Err("没有帧数据".to_string());
    }

    for (i, frame_data_url) in frame_data_urls.iter().enumerate() {
        let frame_b64 = frame_data_url
            .split_once(',')
            .map(|(_, b64)| b64)
            .unwrap_or(frame_data_url);
        let frame_bytes = base64::engine::general_purpose::STANDARD
            .decode(frame_b64)
            .map_err(|_| format!("帧 {} 解码失败", i))?;
        let frame_path = work_dir.join(format!("frame{:06}.png", i));
        fs::write(&frame_path, &frame_bytes)
            .map_err(|e| format!("写入帧 {} 失败: {}", i, e))?;
    }

    let framerate_str = fps.to_string();
    let frame_pattern = work_dir.join("frame%06d.png");
    let frame_pattern_str = frame_pattern.to_string_lossy().to_string();

    run_ffmpeg(&[
        "-y".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-framerate".to_string(),
        framerate_str,
        "-i".to_string(),
        frame_pattern_str,
        "-vf".to_string(),
        format!("fps={}", 30),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-crf".to_string(),
        "18".to_string(),
        "-preset".to_string(),
        "fast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        final_path.to_string_lossy().to_string(),
    ])?;

    if has_audio {
        let audio_path = audio_path.unwrap();
        let start = parse_time_secs(&start_sec).unwrap_or(0.0).max(0.0);
        let end = parse_time_secs(&end_sec).unwrap_or(0.0);

        let audio_trim_path = work_dir.join("audio_trim.aac");
        let mut audio_args = vec![
            "-y".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-i".to_string(),
            audio_path.to_string_lossy().to_string(),
        ];
        if start > 0.0 {
            audio_args.push("-ss".to_string());
            audio_args.push(format!("{:.3}", start));
        }
        if end > 0.0 && end > start {
            audio_args.push("-t".to_string());
            audio_args.push(format!("{:.3}", end - start));
        }
        audio_args.extend([
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "192k".to_string(),
            audio_trim_path.to_string_lossy().to_string(),
        ]);
        run_ffmpeg(&audio_args)?;

        let final_with_audio = work_dir.join("output_with_audio.mp4");
        run_ffmpeg(&[
            "-y".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-i".to_string(),
            final_path.to_string_lossy().to_string(),
            "-i".to_string(),
            audio_trim_path.to_string_lossy().to_string(),
            "-c".to_string(),
            "copy".to_string(),
            "-map".to_string(),
            "0:v".to_string(),
            "-map".to_string(),
            "1:a".to_string(),
            "-shortest".to_string(),
            final_with_audio.to_string_lossy().to_string(),
        ])?;

        fs::rename(&final_with_audio, &final_path)
            .map_err(|e| format!("重命名输出文件失败: {e}"))?;
    }

    let out_dir = get_editor_export_dir("视频")?;
    let out_name = format!("EditorVideo_{}_{}.mp4", sanitize_filename(&title), now_millis());
    let out_path = out_dir.join(&out_name);
    fs::copy(&final_path, &out_path).map_err(|e| format!("复制输出文件失败: {e}"))?;

    let _ = fs::remove_dir_all(&work_dir);
    Ok(out_path.to_string_lossy().to_string())
}

fn get_editor_export_dir(subdir: &str) -> Result<PathBuf, String> {
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| "无法找到桌面目录".to_string())?;
    let export_dir = desktop.join("Editor").join(subdir);
    if !export_dir.exists() {
        fs::create_dir_all(&export_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;
    }
    Ok(export_dir)
}

#[tauri::command]
fn save_image(image_data_url: String, filename: String) -> Result<String, String> {
    let out_dir = get_editor_export_dir("图片")?;
    let out_path = out_dir.join(&filename);
    let b64 = image_data_url
        .split_once(',')
        .map(|(_, b)| b)
        .unwrap_or(&image_data_url);
    let data = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| "图片base64解码失败".to_string())?;
    fs::write(&out_path, &data).map_err(|e| format!("保存图片失败: {}", e))?;
    Ok(out_path.to_string_lossy().to_string())
}

fn run_ffmpeg(args: &[String]) -> Result<String, String> {
    let ffmpeg = get_ffmpeg_cmd().ok_or_else(|| "未找到 FFmpeg".to_string())?;
    let output = Command::new(&ffmpeg)
        .args(args)
        .output()
        .map_err(|e| format!("FFmpeg 执行失败: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![export_video_ffmpeg, inspect_audio_tags, save_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}