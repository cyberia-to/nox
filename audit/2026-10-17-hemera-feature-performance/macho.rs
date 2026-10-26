use std::{env, fs, io, path::Path};

fn u32_at(bytes: &[u8], at: usize) -> io::Result<u32> {
    let raw = bytes.get(at..at + 4).ok_or_else(|| io::Error::other("u32 bounds"))?;
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}
fn u64_at(bytes: &[u8], at: usize) -> io::Result<u64> {
    let raw = bytes.get(at..at + 8).ok_or_else(|| io::Error::other("u64 bounds"))?;
    Ok(u64::from_le_bytes(raw.try_into().unwrap()))
}
fn name(bytes: &[u8]) -> io::Result<String> {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let text = std::str::from_utf8(&bytes[..end]).map_err(io::Error::other)?;
    if text.contains(['\t', '\n', '/', '\\']) {
        return Err(io::Error::other("unsafe Mach-O name"));
    }
    Ok(text.to_owned())
}
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(io::Error::other("usage: macho BINARY NEW_OUTPUT_DIRECTORY"));
    }
    let bytes = fs::read(&args[1])?;
    if u32_at(&bytes, 0)? != 0xfeed_facf || u32_at(&bytes, 4)? != 0x0100_000c {
        return Err(io::Error::other("expected little-endian arm64 Mach-O"));
    }
    let out = Path::new(&args[2]);
    fs::create_dir(out)?;
    let mut table = String::from("segment\tsection\taddress\tfile_offset\tbytes\tflags\tfile_backed\n");
    let mut at = 32usize;
    let end = at + u32_at(&bytes, 20)? as usize;
    let mut text_base = 0u64;
    let mut text_end = 0u64;
    let mut starts_data = None;
    let mut symtab = None;
    for _ in 0..u32_at(&bytes, 16)? {
        let cmd = u32_at(&bytes, at)?;
        let len = u32_at(&bytes, at + 4)? as usize;
        if len < 8 || at + len > end || end > bytes.len() {
            return Err(io::Error::other("load command bounds"));
        }
        if cmd == 0x19 {
            if name(&bytes[at + 8..at + 24])? == "__TEXT" {
                text_base = u64_at(&bytes, at + 24)?;
            }
            let count = u32_at(&bytes, at + 64)? as usize;
            if 72 + count * 80 > len {
                return Err(io::Error::other("section table bounds"));
            }
            for n in 0..count {
                let s = at + 72 + n * 80;
                let section = name(&bytes[s..s + 16])?;
                let segment = name(&bytes[s + 16..s + 32])?;
                let address = u64_at(&bytes, s + 32)?;
                let size = u64_at(&bytes, s + 40)?;
                let offset = u32_at(&bytes, s + 48)? as usize;
                let flags = u32_at(&bytes, s + 64)?;
                if segment == "__TEXT" && section == "__text" {
                    text_end = address + size;
                }
                let backed = !matches!(flags & 0xff, 0x01 | 0x0c | 0x12);
                if backed {
                    let data = bytes.get(offset..offset + size as usize)
                        .ok_or_else(|| io::Error::other("section body bounds"))?;
                    fs::write(out.join(format!("{segment}.{section}.bin")), data)?;
                }
                table.push_str(&format!("{segment}\t{section}\t{address}\t{offset}\t{size}\t{flags}\t{backed}\n"));
            }
        } else if cmd == 0x26 {
            starts_data = Some((u32_at(&bytes, at + 8)? as usize, u32_at(&bytes, at + 12)? as usize));
        } else if cmd == 0x02 {
            symtab = Some((u32_at(&bytes, at + 8)? as usize, u32_at(&bytes, at + 12)? as usize,
                u32_at(&bytes, at + 16)? as usize, u32_at(&bytes, at + 20)? as usize));
        }
        at += len;
    }
    if at != end {
        return Err(io::Error::other("load command length mismatch"));
    }
    fs::write(out.join("sections.tsv"), table)?;
    let mut symbols = Vec::new();
    if let Some((offset, count, strings, string_size)) = symtab {
        if offset + count * 16 > bytes.len() || strings + string_size > bytes.len() {
            return Err(io::Error::other("symbol table bounds"));
        }
        for n in 0..count {
            let s = offset + n * 16;
            let str_index = u32_at(&bytes, s)? as usize;
            if bytes[s + 4] & 0xe == 0xe && str_index < string_size {
                symbols.push((u64_at(&bytes, s + 8)?, name(&bytes[strings + str_index..strings + string_size])?));
            }
        }
    }
    let mut starts = Vec::new();
    if let Some((offset, size)) = starts_data {
        let raw = bytes.get(offset..offset + size).ok_or_else(|| io::Error::other("function starts bounds"))?;
        let mut address = text_base;
        let mut cursor = 0;
        while cursor < raw.len() {
            let mut value = 0u64;
            let mut shift = 0;
            loop {
                let b = *raw.get(cursor).ok_or_else(|| io::Error::other("truncated ULEB128"))?;
                cursor += 1;
                if shift >= 64 { return Err(io::Error::other("ULEB128 overflow")); }
                value |= u64::from(b & 0x7f) << shift;
                if b & 0x80 == 0 { break; }
                shift += 7;
            }
            if value == 0 { break; }
            address = address.checked_add(value).ok_or_else(|| io::Error::other("function address overflow"))?;
            if address >= text_end { return Err(io::Error::other("function outside text")); }
            starts.push(address);
        }
    }
    let mut functions = String::from("address\tspan_to_next_start\tsymbols\n");
    for (i, address) in starts.iter().enumerate() {
        let next = starts.get(i + 1).copied().unwrap_or(text_end);
        let labels = symbols.iter().filter(|(a, _)| a == address).map(|(_, s)| s.as_str()).collect::<Vec<_>>().join(";");
        functions.push_str(&format!("{address}\t{}\t{labels}\n", next - address));
    }
    fs::write(out.join("functions.tsv"), functions)?;
    Ok(())
}
