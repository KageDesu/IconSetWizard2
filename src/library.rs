use crate::document::{decode_png, fit_cell};
use image::RgbaImage;
use std::path::{Path, PathBuf};

pub struct Package {
    pub name: String,
    pub info: String,
    pub path: PathBuf,
    pub items: Vec<RgbaImage>,
}

pub fn load_folder(folder: &Path) -> Result<(Vec<Package>, Vec<String>), String> {
    let mut paths = std::fs::read_dir(folder)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("iwpk"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    let mut packages = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match load_package(&path) {
            Ok(package) => packages.push(package),
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    Ok((packages, errors))
}

pub fn load_package(path: &Path) -> Result<Package, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Пакет превышает 32 MB".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let text = if bytes.starts_with(&[0xff, 0xfe]) {
        if !bytes.len().is_multiple_of(2) {
            return Err("Некорректный UTF-16".into());
        }
        let words = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<_>>();
        String::from_utf16(&words).map_err(|e| e.to_string())?
    } else {
        String::from_utf8(bytes).map_err(|e| e.to_string())?
    };
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let name = lines.next().ok_or("Нет имени категории")?.to_owned();
    let info = lines.next().ok_or("Нет строки описания")?.to_owned();
    let mut items = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if items.len() >= 8192 {
            return Err("Слишком много иконок в пакете".into());
        }
        let decoded = line
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let value = (c as u32)
                    .checked_sub(b"iwpk"[i % 4] as u32)
                    .ok_or("Некорректная кодировка пакета")?;
                u8::try_from(value).map_err(|_| "Некорректная кодировка пакета")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let image = decode_base64(&decoded)
            .and_then(|bytes| decode_png(&bytes))
            .map_err(|e| format!("Иконка {}: {e}", index + 1))?;
        items.push(fit_cell(image));
    }
    if items.is_empty() {
        return Err("Пакет не содержит иконок".into());
    }
    Ok(Package {
        name,
        info,
        path: path.to_owned(),
        items,
    })
}

fn decode_base64(input: &[u8]) -> Result<Vec<u8>, String> {
    let input = input
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if !input.len().is_multiple_of(4) {
        return Err("Некорректная длина Base64".into());
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    for (index, chunk) in input.as_chunks::<4>().0.iter().enumerate() {
        let mut values = [0u32; 4];
        let mut padding = 0;
        for (i, b) in chunk.iter().enumerate() {
            values[i] = match b {
                b'A'..=b'Z' if padding == 0 => (b - b'A') as u32,
                b'a'..=b'z' if padding == 0 => (b - b'a' + 26) as u32,
                b'0'..=b'9' if padding == 0 => (b - b'0' + 52) as u32,
                b'+' if padding == 0 => 62,
                b'/' if padding == 0 => 63,
                b'=' if i >= 2 && index == input.len() / 4 - 1 => {
                    padding += 1;
                    0
                }
                _ => return Err("Некорректный Base64".into()),
            };
        }
        let value = values[0] << 18 | values[1] << 12 | values[2] << 6 | values[3];
        out.push((value >> 16) as u8);
        if padding < 2 {
            out.push((value >> 8) as u8);
        }
        if padding == 0 {
            out.push(value as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn base64_known_values_and_invalid_padding() {
        assert_eq!(decode_base64(b"TWFu").unwrap(), b"Man");
        assert_eq!(decode_base64(b"TWE=").unwrap(), b"Ma");
        assert_eq!(decode_base64(b"TQ==").unwrap(), b"M");
        assert!(decode_base64(b"T=Fu").is_err());
    }
    #[test]
    fn shipped_packages_decode() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("packages");
        let (packages, errors) = load_folder(&folder).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        assert!(packages.len() >= 10);
        println!(
            "Decoded {} packages, {} icons",
            packages.len(),
            packages.iter().map(|p| p.items.len()).sum::<usize>()
        );
        assert!(packages.iter().all(|p| !p.items.is_empty()));
        assert!(packages.iter().any(|p| p.name == "MV Battlers"));
    }
}
