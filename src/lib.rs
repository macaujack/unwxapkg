use std::io::Read;

pub struct MiniappFile {
    /// 文件的完整路徑，比如： /pages/webview/webview.html
    pub filename: String,

    /// 文件的二進制內容
    pub content: Vec<u8>,
}

/// 根據給定的 wxapkg 格式的文件來解碼
pub fn decode_wxapkg<T: Read>(file: &mut T) -> Result<Vec<MiniappFile>, String> {
    let mut wxapkg = Vec::new();
    if let Err(e) = file.read_to_end(&mut wxapkg) {
        return Err(format!("Error reading file: {}", e.to_string()));
    }
    let wxapkg = wxapkg;

    // 讀 header
    let _ = convert_to_header(&wxapkg[0..14])?;

    // 讀文件數量
    let num_files = convert_to_u32(&wxapkg[14..18]);

    // 讀每個文件的內容
    let mut files = Vec::with_capacity(num_files as usize);
    let mut start_index = 18;
    for _ in 0..num_files {
        let (file, next_index) = convert_to_miniapp_file(&wxapkg, start_index)?;
        start_index = next_index;
        files.push(file);
    }

    Ok(files)
}

#[derive(Debug, Clone, Copy)]
struct WxapkgHeader {
    _len_index: u32,
    _len_data: u32,
}

/// 讀 wxapkg 格式的 header，必須保證傳進來的 bytes 的大小爲 14
fn convert_to_header(buf: &[u8]) -> Result<WxapkgHeader, String> {
    // 檢查首尾的 magic number
    if buf[0] != 0xBE || buf[13] != 0xED {
        const ERR_MSG: &str =
            "Incorrect format. The 1st byte should be 0xBE and 13th byte should be 0xED";
        return Err(String::from(ERR_MSG));
    }

    let buf = &buf[1..13];
    // 檢查 padding
    for i in 0..4 {
        if buf[i] != 0 {
            const ERR_MSG: &str = "Incorrect padding. Zero padding (i.e., 4 zeroes) is expected.";
            return Err(String::from(ERR_MSG));
        }
    }

    let ret = WxapkgHeader {
        _len_index: convert_to_u32(&buf[4..8]),
        _len_data: convert_to_u32(&buf[8..12]),
    };

    Ok(ret)
}

fn convert_to_miniapp_file(
    wxapkg: &[u8],
    start_index: usize,
) -> Result<(MiniappFile, usize), String> {
    let len_filename = convert_to_u32(&wxapkg[start_index..start_index + 4]) as usize;
    let filename = &wxapkg[start_index + 4..start_index + 4 + len_filename];
    let filename = match std::str::from_utf8(filename) {
        Ok(v) => v.to_string(),
        Err(e) => return Err(format!("Incorrect UTF-8 filename: {}", e.to_string())),
    };

    let base = start_index + 4 + len_filename;
    let offset = convert_to_u32(&wxapkg[base..base + 4]) as usize;
    let content_size = convert_to_u32(&wxapkg[base + 4..base + 8]) as usize;
    let content = &wxapkg[offset..offset + content_size];

    Ok((
        MiniappFile {
            filename,
            content: Vec::from(content),
        },
        base + 8,
    ))
}

/// 將 4 個 byte 轉為 u32（大端），必須保證 bytes 的大小為 4
fn convert_to_u32(bytes: &[u8]) -> u32 {
    let mut ret = 0;

    for i in 0..4 {
        ret = (ret << 8) | (bytes[i] as u32);
    }

    ret
}
