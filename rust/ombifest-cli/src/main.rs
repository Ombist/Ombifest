//! ombifest — Rust CLI (Swift-compatible). See Ombifest/SPEC.md

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use clap::{Parser, Subcommand};
use ombifest_cli::canonical::normalize_pins_from_comma_separated;
use ombifest_cli::crypto::generate_key_pair_pkcs8_pem;
use ombifest_cli::leaf_pin::leaf_pin_from_cert_pem;
use ombifest_cli::manifest::{normalize_pins_from_csv, sign_manifest_object, verify_manifest};
use ombifest_cli::manifest_file::{read_manifest_utf8_limited, DEFAULT_MAX_MANIFEST_BYTES};

#[derive(Parser)]
#[command(name = "ombifest")]
#[command(about = "TLS leaf pin & signed manifest (Ombist iOS compatible)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sign manifest JSON to stdout
    Sign {
        #[arg(long)]
        pins: String,
        #[arg(long = "valid-until")]
        valid_until: String,
        #[arg(long)]
        version: i64,
        #[arg(long = "private-key")]
        private_key: PathBuf,
    },
    /// Verify manifest file
    Verify {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long = "public-key-hex")]
        public_key_hex: String,
        #[arg(long)]
        now: Option<String>,
    },
    /// Generate Ed25519 keypair (PKCS#8 PEM)
    GenerateKey {
        #[arg(long = "out-private")]
        out_private: Option<PathBuf>,
        #[arg(long = "print-private-to-stdout", action = clap::ArgAction::SetTrue)]
        print_private_to_stdout: bool,
    },
    /// Leaf cert DER SHA-256 hex (64 lowercase hex)
    LeafPin {
        #[arg(long)]
        cert: PathBuf,
    },
    /// Build relay manifest from leaf cert + optional dual pin
    BuildRelay {
        #[arg(long = "leaf-cert")]
        leaf_cert: PathBuf,
        #[arg(long = "valid-until")]
        valid_until: String,
        #[arg(long)]
        version: i64,
        #[arg(long = "private-key")]
        private_key: PathBuf,
        #[arg(long = "next-pin")]
        next_pin: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(code) = run(cli) {
        std::process::exit(code);
    }
}

fn run(cli: Cli) -> Result<(), i32> {
    match cli.command {
        Commands::Sign {
            pins,
            valid_until,
            version,
            private_key,
        } => {
            let pins_v = normalize_pins_from_comma_separated(&pins);
            let pem = fs::read_to_string(&private_key).map_err(|e| {
                eprintln!("sign failed: {e}");
                1
            })?;
            let manifest = sign_manifest_object(&pins_v, &valid_until, version, &pem).map_err(|e| {
                eprintln!("sign failed: {e}");
                1
            })?;
            println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
            Ok(())
        }
        Commands::Verify {
            manifest,
            public_key_hex,
            now,
        } => {
            let body = read_manifest_utf8_limited(&manifest, DEFAULT_MAX_MANIFEST_BYTES).map_err(|e| {
                eprintln!("verify failed: {e}");
                1
            })?;
            let ref_dt = if let Some(n) = now {
                n.parse::<chrono::DateTime<Utc>>()
                    .map_err(|e| {
                        eprintln!("verify failed: bad --now: {e}");
                        1
                    })?
            } else {
                Utc::now()
            };
            let pins = verify_manifest(&body, &public_key_hex, ref_dt).map_err(|e| {
                eprintln!("verify failed: {e}");
                1
            })?;
            eprintln!("ok: {} pin(s)", pins.len());
            Ok(())
        }
        Commands::GenerateKey {
            out_private,
            print_private_to_stdout,
        } => {
            if !print_private_to_stdout && out_private.is_none() {
                eprintln!(
                    "generate-key: specify --out-private <path.pem> (recommended) or --print-private-to-stdout (unsafe)."
                );
                return Err(1);
            }
            let rng = ring::rand::SystemRandom::new();
            let (pem, pub_hex) = generate_key_pair_pkcs8_pem(&rng).map_err(|e| {
                eprintln!("generate-key failed: {e}");
                1
            })?;
            if print_private_to_stdout {
                eprintln!("ombifest: warning: private key PEM is printed to stdout; prefer --out-private.");
                println!("--- PRIVATE PEM (protect offline) ---");
                print!("{pem}");
                println!("\nOMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX (64 hex chars):");
                println!("{pub_hex}");
            } else {
                let path = out_private.unwrap();
                write_private_pem(&path, &pem).map_err(|e| {
                    eprintln!("generate-key failed: {e}");
                    1
                })?;
                eprintln!(
                    "Wrote private key to {} (mode 0600 where supported).",
                    path.display()
                );
                eprintln!("OMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX (64 hex chars):");
                println!("{pub_hex}");
            }
            Ok(())
        }
        Commands::LeafPin { cert } => {
            let pin = leaf_pin_from_cert_pem(&cert).map_err(|e| {
                eprintln!("{e}");
                1
            })?;
            if !pin.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) || pin.len() != 64 {
                eprintln!("failed to compute leaf pin");
                return Err(1);
            }
            println!("{pin}");
            Ok(())
        }
        Commands::BuildRelay {
            leaf_cert,
            valid_until,
            version,
            private_key,
            next_pin,
        } => {
            let current = leaf_pin_from_cert_pem(&leaf_cert).map_err(|e| {
                eprintln!("{e}");
                1
            })?;
            let current = current.trim().to_lowercase();
            if current.len() != 64 || !current.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
                eprintln!("failed to compute current leaf pin");
                return Err(1);
            }
            let pins_csv = if let Some(np) = next_pin {
                let np = np.trim().to_lowercase();
                if np.len() != 64 || !np.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
                    eprintln!("--next-pin must be 64 hex chars");
                    return Err(1);
                }
                format!("{current},{np}")
            } else {
                current.clone()
            };
            let pins = normalize_pins_from_csv(&pins_csv);
            let pem = fs::read_to_string(&private_key).map_err(|e| {
                eprintln!("build-relay failed: {e}");
                1
            })?;
            let manifest = sign_manifest_object(&pins, &valid_until, version, &pem).map_err(|e| {
                eprintln!("build-relay failed: {e}");
                1
            })?;
            println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
            Ok(())
        }
    }
}

fn write_private_pem(path: &PathBuf, pem: &str) -> std::io::Result<()> {
    use std::io::Write;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(pem.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        fs::write(path, pem)?;
    }
    Ok(())
}
