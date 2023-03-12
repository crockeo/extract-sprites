use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::anyhow;
use image::Rgb;
use image::RgbImage;

fn main() -> anyhow::Result<()> {
    let mut cpair = CPair::default();
    let mut zip_archive = zip::ZipArchive::new(File::open("pbobblen.zip")?)?;
    for i in 0..zip_archive.len() {
        let mut zip_file = zip_archive.by_index(i)?;
        if !zip_file.is_file() {
            continue;
        }

        let name = zip_file.name().to_string();
        let Some((name, ext)) = name.rsplit_once(".") else {
	    continue;
	};
        if !ext.starts_with("c") {
            continue;
        }

        let mut contents = Vec::new();
        zip_file.read_to_end(&mut contents)?;

        if contents.len() % CPAIR_CHUNK_WIDTH != 0 {
            return Err(anyhow!(
                "Contents of `{}.{}` is not a multiple of {} bytes :(",
                name,
                ext,
                CPAIR_CHUNK_WIDTH,
            ));
        }

        if ext == "c5" {
            cpair.odd = contents;
        } else if ext == "c6" {
            cpair.even = contents;
        } else {
            todo!("implement general C ROM handling");
        }
    }

    let palette = make_palette();
    for (i, chunk) in cpair.chunks().enumerate() {
        let sprite = chunk.parse_sprite();

        let mut img = RgbImage::new(16, 16);
        for col in 0..16 {
            for row in 0..16 {
                let index = (row * 16 + col) as usize;
                img.put_pixel(col, row, palette[sprite[index] as usize])
            }
        }

        let mut filename = PathBuf::from("images");
        filename.push(format!("sprite_{}.png", i));
        eprintln!("Saving {}...", filename.to_str().unwrap());
        img.save(filename)?;
    }

    Ok(())
}

fn make_palette() -> [Rgb<u8>; 16] {
    let step = 256 / 7;
    let mut colors = [Rgb([0, 0, 0]); 16];
    for i in 0..16 {
        let value = (i + 1) * step;

        let red = (value as f32 * 0.6) as u8;
        let green = (value as f32 * 0.9) as u8;
        let blue = value as u8;
        colors[i] = Rgb([red, green, blue]);
    }
    colors
}

const CPAIR_CHUNK_WIDTH: usize = 64;

#[derive(Default)]
struct CPair {
    even: Vec<u8>,
    odd: Vec<u8>,
}

impl CPair {
    fn chunks(&self) -> CPairChunkIterator<'_> {
        CPairChunkIterator {
            cpair: self,
            index: 0,
        }
    }
}

struct CPairChunkIterator<'a> {
    cpair: &'a CPair,
    index: usize,
}

impl<'a> Iterator for CPairChunkIterator<'a> {
    type Item = CPairChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.index * CPAIR_CHUNK_WIDTH;
        let end = (self.index + 1) * CPAIR_CHUNK_WIDTH;

        if start >= self.cpair.even.len() || start >= self.cpair.odd.len() {
            return None;
        }

        let next = Some(CPairChunk {
            even: &self.cpair.even[start..end],
            odd: &self.cpair.odd[start..end],
        });
        self.index += 1;
        next
    }
}

struct CPairChunk<'a> {
    even: &'a [u8],
    odd: &'a [u8],
}

impl CPairChunk<'_> {
    fn parse_sprite(&self) -> [u8; 256] {
        let mut pixels = [0; 256];
        for i in 0..32 {
            let bp0 = self.odd[i * 2];
            let bp1 = self.odd[i * 2 + 1];
            let bp2 = self.even[i * 2];
            let bp3 = self.even[i * 2 + 1];

            for bit in 0..8 {
                let mut index = 0;
                index |= (bp0 >> bit) & 1;
                index |= ((bp1 >> bit) & 1) << 1;
                index |= ((bp2 >> bit) & 1) << 2;
                index |= ((bp3 >> bit) & 1) << 3;
                pixels[i * 8 + bit] = index
            }
        }
	pixels
    }
}
