// Probe: verifica que DejaVuSans.ttf carga y tiene glyphs ASCII
use ab_glyph::{Font, FontVec};

fn main() {
    let candidates = [
        "/run/current-system/sw/share/fonts/DejaVuSans.ttf",
        "/nix/store/ang6yzsv32vnkdq7bqr41dgna2knkz8w-dejavu-fonts-minimal-2.37/share/fonts/truetype/DejaVuSans.ttf",
    ];
    for path in candidates {
        match std::fs::read(path) {
            Ok(data) => {
                println!("{}: {} bytes", path, data.len());
                match FontVec::try_from_vec(data) {
                    Ok(font) => {
                        for ch in ['H', 'e', 'o', '[', 'D', '/', 'a'] {
                            let gid = font.glyph_id(ch);
                            println!("  '{}' -> gid {}", ch, gid.0);
                        }
                        println!("  units_per_em: {:?}", font.units_per_em());
                    }
                    Err(e) => println!("  parse ERROR: {:?}", e),
                }
            }
            Err(e) => println!("{}: read ERROR {}", path, e),
        }
    }
}