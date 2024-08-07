use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};

use bytesize::ByteSize;
use clap::Parser;
use memmem::{Searcher, TwoWaySearcher};
use tracing::{debug, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

/// From `libil2cpp.sp`:
/// ```
/// 6dNRoG04n56HX2LiOAkpCC9fgjxvMKDyZGyx35Owh/sOU8HjpOdGHBy96ytzw9WMxzyvJkl
/// 29Q82mc4y7zKy3SkchVC16mnckCO26kf6Wn4Xe5lN0i7Ot5kIueWY2ioo8iRudj/EbNdumT
/// U8I7oC7dWuvIEovK4eDJdFJO2tzZ8=
/// ```
pub static ORIGINAL_KEY: &[u8] = &[
  0xe9, 0xd3, 0x51, 0xa0, 0x6d, 0x38, 0x9f, 0x9e, 0x87, 0x5f, 0x62, 0xe2, 0x38, 0x09, 0x29, 0x08, 0x2f, 0x5f, 0x82,
  0x3c, 0x6f, 0x30, 0xa0, 0xf2, 0x64, 0x6c, 0xb1, 0xdf, 0x93, 0xb0, 0x87, 0xfb, 0x0e, 0x53, 0xc1, 0xe3, 0xa4, 0xe7,
  0x46, 0x1c, 0x1c, 0xbd, 0xeb, 0x2b, 0x73, 0xc3, 0xd5, 0x8c, 0xc7, 0x3c, 0xaf, 0x26, 0x49, 0x76, 0xf5, 0x0f, 0x36,
  0x99, 0xce, 0x32, 0xef, 0x32, 0xb2, 0xdd, 0x29, 0x1c, 0x85, 0x50, 0xb5, 0xea, 0x69, 0xdc, 0x90, 0x23, 0xb6, 0xea,
  0x47, 0xfa, 0x5a, 0x7e, 0x17, 0x7b, 0x99, 0x4d, 0xd2, 0x2e, 0xce, 0xb7, 0x99, 0x08, 0xb9, 0xe5, 0x98, 0xda, 0x2a,
  0x28, 0xf2, 0x24, 0x6e, 0x76, 0x3f, 0xc4, 0x6c, 0xd7, 0x6e, 0x99, 0x35, 0x3c, 0x23, 0xba, 0x02, 0xed, 0xd5, 0xae,
  0xbc, 0x81, 0x28, 0xbc, 0xae, 0x1e, 0x0c, 0x97, 0x45, 0x24, 0xed, 0xad, 0xcd, 0x9f,
];

/// Run `openssl genpkey -algorithm RSA -out key.pem -pkeyopt rsa_keygen_bits:1024` to generate key.
///
/// Run `openssl rsa -in private_key.pem -pubout -text -noout` and copy modulus.
pub static NEW_KEY: &[u8] = &[
  0xcf, 0xb5, 0x5e, 0x36, 0xcb, 0x98, 0x2d, 0x7f, 0xff, 0x73, 0x1f, 0xe8, 0x33, 0x11, 0x0c, 0xc4, 0xa8, 0x2f, 0xa3,
  0x8f, 0xf8, 0xc2, 0xeb, 0x01, 0xdd, 0x06, 0xd5, 0xdd, 0x92, 0xf0, 0xeb, 0x50, 0x08, 0x0a, 0x53, 0x52, 0x8e, 0xf3,
  0x0d, 0x6c, 0x81, 0x41, 0xa5, 0x9c, 0x49, 0x27, 0xd0, 0x22, 0x9f, 0x21, 0x97, 0x7b, 0xa0, 0xb5, 0x96, 0x6a, 0x82,
  0x85, 0xfa, 0xe5, 0x84, 0x1b, 0x4e, 0x0f, 0x06, 0x6e, 0x82, 0xa2, 0x9e, 0x15, 0xa0, 0x0c, 0x44, 0xdf, 0x54, 0x1a,
  0x50, 0xd1, 0xc9, 0x29, 0x21, 0x25, 0x99, 0x0c, 0xc9, 0xeb, 0x9f, 0x53, 0xc9, 0x01, 0x04, 0x4b, 0xc7, 0x93, 0x8e,
  0x34, 0x28, 0xcb, 0xd8, 0x4e, 0x00, 0x97, 0x6f, 0x06, 0x09, 0x2a, 0x3a, 0x97, 0xc8, 0xcf, 0x4a, 0x3a, 0x05, 0xe7,
  0xa3, 0xc3, 0xba, 0x83, 0x73, 0x84, 0xb0, 0xb5, 0xc8, 0xc0, 0x63, 0x1e, 0xbe, 0x0d,
];

pub static ORIGINAL_STATIC_URL: &str = "https://static-prd-wonder.sesisoft.com/";

#[derive(Parser, Debug)]
struct Args {
  /// Publicly accessible URL of the static server. (e.g. "https://static.yourdomain.dev/")
  #[arg(long)]
  url: Option<String>,

  /// PID of the `com.nexon.konosuba` process.
  pid: u32,
}

#[derive(Debug)]
pub struct MemoryRegion {
  pub start: usize,
  pub end: usize,
  pub offset: usize,
  pub perms: String,
  pub inode: String,
  pub dev: String,
  pub pathname: Option<String>,
}

impl MemoryRegion {
  pub fn size(&self) -> usize {
    self.end - self.start
  }
}

fn read_memory_regions(pid: u32) -> io::Result<Vec<MemoryRegion>> {
  let path = format!("/proc/{}/maps", pid);
  let file = File::open(&path)?;
  let reader = io::BufReader::new(file);

  let mut regions = Vec::new();

  for line in reader.lines() {
    let line = line?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
      continue;
    }

    debug!("{:?}", parts);
    let (start, end) = parts[0].split_once('-').unwrap();
    let (start, end) = (
      usize::from_str_radix(start, 16).unwrap(),
      usize::from_str_radix(end, 16).unwrap(),
    );
    let perms = parts[1].to_string();
    let offset = usize::from_str_radix(parts[2], 16).unwrap();
    let dev = parts[3].to_string();
    let inode = parts[4].to_string();
    let pathname = if parts.len() > 5 {
      Some(parts[5..].join(" "))
    } else {
      None
    };

    regions.push(MemoryRegion {
      start,
      end,
      offset,
      perms,
      dev,
      inode,
      pathname,
    });
  }

  Ok(regions)
}

fn str_to_utf16_bytes(input: &str) -> Vec<u8> {
  let utf16: Vec<u16> = input.encode_utf16().collect();
  let mut utf16_bytes = Vec::with_capacity(utf16.len() * 2);

  for word in utf16 {
    utf16_bytes.push((word >> 8) as u8);
    utf16_bytes.push((word & 0xFF) as u8);
  }

  utf16_bytes
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();

  let args = Args::parse();

  assert_eq!(ORIGINAL_KEY.len(), NEW_KEY.len());

  let new_static_url = if let Some(new_static_url) = &args.url {
    if new_static_url.len() > ORIGINAL_STATIC_URL.len() {
      error!(
        "Static URL ({:?}) - {} characters is longer than original: {} characters",
        new_static_url,
        new_static_url.len(),
        ORIGINAL_STATIC_URL.len()
      );
      std::process::exit(1);
    }
    let new_static_url = format!("{:/<width$}", new_static_url, width = ORIGINAL_STATIC_URL.len());
    info!("padded static url: {}", new_static_url);

    Some(new_static_url)
  } else {
    None
  };

  let regions = match read_memory_regions(args.pid) {
    Ok(regions) => regions,
    Err(error) => {
      error!("Error reading memory regions: {}", error);
      std::process::exit(1);
    }
  };

  let mut suitable_regions = Vec::new();
  for region in &regions {
    if !region.perms.contains("w") {
      debug!(?region, "region is not writable");
      continue;
    }

    suitable_regions.push(region);
  }

  let total_size = ByteSize::b(suitable_regions.iter().map(|region| region.size() as u64).sum());
  info!("{} to scan", total_size);

  let mut found_url = 0;
  let mut found_rsa = 0;

  for region in &suitable_regions {
    let size = ByteSize::b(region.size() as u64);
    debug!("searching {:?}: {}", region, size);

    if let Some(new_static_url) = &new_static_url {
      let addresses = match search_byte_sequence(args.pid, &region, &str_to_utf16_bytes(ORIGINAL_STATIC_URL)) {
        Ok(addresses) => addresses,
        Err(error) => {
          warn!("Error searching memory region {:?}: {}", region, error);
          continue;
        }
      };

      for address in &addresses {
        info!(
          "Found domain sequence at address: 0x{:x} in region {:?}",
          address, region
        );

        let virtual_address = region.start + address;
        match write_to_memory(args.pid, virtual_address, &str_to_utf16_bytes(new_static_url)) {
          Ok(_) => {
            info!("Successfully wrote to memory at address 0x{:x}", virtual_address);
            found_url += 1;
          }
          Err(error) => error!("Error writing to memory: {}", error),
        }
      }
    }

    {
      let addresses = match search_byte_sequence(
        args.pid,
        &region,
        &ORIGINAL_KEY.iter().cloned().rev().collect::<Vec<_>>(),
      ) {
        Ok(addresses) => addresses,
        Err(error) => {
          warn!("Error searching memory region {:?}: {}", region, error);
          continue;
        }
      };

      for address in &addresses {
        info!(
          "Found RSA key sequence at address: 0x{:x} in region {:?}",
          address, region
        );

        let virtual_address = region.start + address;
        match write_to_memory(
          args.pid,
          virtual_address,
          &NEW_KEY.iter().cloned().rev().collect::<Vec<_>>(),
        ) {
          Ok(_) => {
            info!("Successfully wrote to memory at address 0x{:x}", virtual_address);
            found_rsa += 1;
          }
          Err(error) => error!("Error writing to memory: {}", error),
        }
      }
    }
  }

  if found_url > 0 {
    info!(pid = ?args.pid, "Replaced static URL {} times", found_url);
  } else {
    error!(pid = ?args.pid, "No static URL key found in memory");
  }

  if found_rsa > 0 {
    info!(pid = ?args.pid, "Replaced RSA key {} times", found_rsa);
  } else {
    error!(pid = ?args.pid, "No RSA key found in memory");
    error!("Possible causes:");
    error!("- You have already replaced the key in this process");
    error!("- The client has failed to connect to the API server (RSA is created lazily on first response)");
  }

  if found_url < 1 || found_rsa < 1 {
    std::process::exit(1);
  }

  Ok(())
}

fn search_byte_sequence(pid: u32, region: &MemoryRegion, sequence: &[u8]) -> io::Result<Vec<usize>> {
  let mem_path = format!("/proc/{}/mem", pid);
  let mut mem_file = OpenOptions::new().read(true).open(&mem_path)?;

  let mut buffer = vec![0; region.size()];
  mem_file.seek(SeekFrom::Start(region.start as u64))?;
  let read = mem_file.read(&mut buffer)?;
  debug!("read {} / {} bytes", read, buffer.len());

  let mut start = 0;
  let mut addresses = Vec::new();
  let searcher = TwoWaySearcher::new(sequence);
  loop {
    debug!("searching from {}", start);
    let position = searcher.search_in(&buffer[start..]);
    if let Some(position) = position {
      addresses.push(start + position);
      start += position + sequence.len();
    } else {
      break;
    }
  }

  Ok(addresses)
}

fn write_to_memory(pid: u32, address: usize, data: &[u8]) -> io::Result<()> {
  let mem_path = format!("/proc/{}/mem", pid);
  let mut mem_file = OpenOptions::new().read(true).write(true).open(&mem_path)?;

  mem_file.seek(SeekFrom::Start(address as u64))?;
  mem_file.write_all(data)?;

  Ok(())
}
