use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, stdin, stdout, BufRead, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use bytesize::ByteSize;
use clap::Parser;
use memmem::{Searcher, TwoWaySearcher};
use openssl::rsa::Rsa;
use tracing::{debug, enabled, error, info, trace, warn, Level};
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

pub static ORIGINAL_STATIC_URL: &str = "https://static-prd-wonder.sesisoft.com/";

#[derive(Parser, Debug)]
struct Args {
  /// Publicly accessible URL of the static server. (e.g. "https://static.yourdomain.dev/")
  #[arg(long)]
  url: Option<String>,

  /// PID of the `com.nexon.konosuba` process.
  pid: u32,

  /// PEM-formatted RSA-1024 key for JWT
  key_file: PathBuf,
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

    trace!("{:?}", parts);
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

fn read_public_key(key_file: &Path) -> Vec<u8> {
  let mut pem_file = File::open(key_file).unwrap();
  let mut pem_contents = String::new();
  pem_file.read_to_string(&mut pem_contents).unwrap();

  match Rsa::public_key_from_pem(pem_contents.as_bytes()) {
    Ok(rsa) => {
      info!("read public key from {:?}", key_file);
      rsa.n().to_vec()
    }
    Err(error) => {
      warn!("Rsa::public_key_from_pem error: {}, treating as private key", error);
      match Rsa::private_key_from_pem(pem_contents.as_bytes()) {
        Ok(rsa) => {
          info!("read private key from {:?}", key_file);
          rsa.n().to_vec()
        }
        Err(error) => {
          error!("Rsa::private_key_from_pem error: {}", error);
          std::process::exit(1);
        }
      }
    }
  }
}

fn get_suitable_regions(pid: u32) -> Vec<MemoryRegion> {
  let regions = match read_memory_regions(pid) {
    Ok(regions) => regions,
    Err(error) => {
      error!("Error reading memory regions: {}", error);
      std::process::exit(1);
    }
  };

  let mut suitable_regions = Vec::new();
  for region in regions {
    if !region.perms.contains("w") {
      trace!(?region, "region is not writable");
      continue;
    }

    suitable_regions.push(region);
  }

  let total_size = ByteSize::b(suitable_regions.iter().map(|region| region.size() as u64).sum());
  info!("{} to scan", total_size);

  suitable_regions
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();

  let args = Args::parse();

  let new_key = read_public_key(&args.key_file);
  info!("new key: {:?}", new_key);

  assert_eq!(ORIGINAL_KEY.len(), new_key.len());

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

  loop {
    let mut found_original = 0;
    let mut found_patched = 0;
    for region in get_suitable_regions(args.pid) {
      let size = ByteSize::b(region.size() as u64);
      trace!("searching {:?} for static URL: {}", region, size);

      if let Some(new_static_url) = &new_static_url {
        {
          let addresses = match search_byte_sequence(args.pid, &region, &str_to_utf16_bytes(new_static_url)) {
            Ok(addresses) => addresses,
            Err(error) => {
              warn!("Error searching memory region {:?}: {}", region, error);
              continue;
            }
          };

          for address in &addresses {
            debug!(
              "Found patched domain sequence at address: 0x{:x} in region {:?}",
              address, region
            );
            found_patched += 1;
          }
        }

        let addresses = match search_byte_sequence(args.pid, &region, &str_to_utf16_bytes(ORIGINAL_STATIC_URL)) {
          Ok(addresses) => addresses,
          Err(error) => {
            warn!("Error searching memory region {:?}: {}", region, error);
            continue;
          }
        };

        for address in &addresses {
          debug!(
            "Found domain sequence at address: 0x{:x} in region {:?}",
            address, region
          );

          let virtual_address = region.start + address;
          match write_to_memory(args.pid, virtual_address, &str_to_utf16_bytes(new_static_url)) {
            Ok(_) => {
              debug!("Successfully wrote to memory at address 0x{:x}", virtual_address);
              found_original += 1;
            }
            Err(error) => error!("Error writing to memory: {}", error),
          }
        }
      }
    }

    if found_original > 0 {
      info!("Replaced static URL {} times", found_original);
      break;
    }

    if found_patched > 0 {
      info!(
        "Found patched static URL {} times, assuming as already patched",
        found_patched
      );
      break;
    }

    warn!("No static URL key found in memory");
    info!("Press play button and wait for an error message saying \"The game is experiencing reduced performance. Please try again later.\"");
    // if !enabled!(Level::DEBUG) {
    //   info!("Hint: Set RUST_LOG=debug if you believe that you are doing everything correctly");
    // }

    write!(stdout(), "Press Enter to continue...").unwrap();
    stdout().flush().unwrap();
    stdin().read_line(&mut String::new()).unwrap();
  }

  info!(
    "Now we need to replace RSA public key in the process memory, it is loaded after first successful server response"
  );
  loop {
    info!("Press play button and wait for an almost instantaneous error message saying \"An error has occurred. Returning to the Title screen...\"");

    write!(stdout(), "Press Enter to continue...").unwrap();
    stdout().flush().unwrap();
    stdin().read_line(&mut String::new()).unwrap();

    let mut found_original = 0;
    let mut found_patched = 0;
    for region in get_suitable_regions(args.pid) {
      let size = ByteSize::b(region.size() as u64);
      trace!("searching {:?} for RSA key: {}", region, size);

      {
        let addresses =
          match search_byte_sequence(args.pid, &region, &new_key.iter().cloned().rev().collect::<Vec<_>>()) {
            Ok(addresses) => addresses,
            Err(error) => {
              warn!("Error searching memory region {:?}: {}", region, error);
              continue;
            }
          };

        for address in &addresses {
          debug!(
            "Found patched RSA key sequence at address: 0x{:x} in region {:?}",
            address, region
          );
          found_patched += 1;
        }
      }

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
        debug!(
          "Found RSA key sequence at address: 0x{:x} in region {:?}",
          address, region
        );

        let virtual_address = region.start + address;
        match write_to_memory(
          args.pid,
          virtual_address,
          &new_key.iter().cloned().rev().collect::<Vec<_>>(),
        ) {
          Ok(_) => {
            debug!("Successfully wrote to memory at address 0x{:x}", virtual_address);
            found_original += 1;
          }
          Err(error) => error!("Error writing to memory: {}", error),
        }
      }
    }

    if found_original > 0 {
      info!("Replaced RSA key {} times", found_original);
      break;
    }

    if found_patched > 0 {
      info!(
        "Found patched RSA key {} times, assuming as already patched",
        found_patched
      );
      break;
    }

    warn!("No RSA key found in memory");
    info!("Make sure the client has successfully connected to the API server, RSA key is loaded after first successful server response");
    // if !enabled!(Level::DEBUG) {
    //   info!("Hint: Set RUST_LOG=debug if you believe that you are doing everything correctly");
    // }
  }

  Ok(())
}

fn search_byte_sequence(pid: u32, region: &MemoryRegion, sequence: &[u8]) -> io::Result<Vec<usize>> {
  let mem_path = format!("/proc/{}/mem", pid);
  let mut mem_file = OpenOptions::new().read(true).open(&mem_path)?;

  let mut buffer = vec![0; region.size()];
  mem_file.seek(SeekFrom::Start(region.start as u64))?;
  let read = mem_file.read(&mut buffer)?;
  trace!("read {} / {} bytes", read, buffer.len());

  let mut start = 0;
  let mut addresses = Vec::new();
  let searcher = TwoWaySearcher::new(sequence);
  loop {
    trace!("searching from {}", start);
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
