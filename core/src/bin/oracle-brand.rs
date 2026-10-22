//! `oracle-brand`: the Brand Kit from the command line, for scripts and the reproducible demo.
//!
//! ```text
//! oracle-brand new DIR "Nom de marque"
//! oracle-brand add-logo DIR FILE --role primaryLogo [--variant color] [--label "…"]
//! oracle-brand add-color DIR "Nom" #RRGGBB
//! oracle-brand add-font DIR "Famille" "Usage"
//! oracle-brand status DIR
//! oracle-brand plan DIR [--assets id,id] [--presets id,id]
//! oracle-brand export DIR [--assets id,id] [--presets id,id] [--changed]
//! oracle-brand deliver DIR FOLDER
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use oracle_brand::export;
use oracle_brand::model::{Asset, BrandColor, BrandFont, Role, Variant, parse_hex};
use oracle_brand::project::{Project, SourceState};
use oracle_brand::render::Engines;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("oracle-brand : {e}");
            ExitCode::FAILURE
        }
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn ids(args: &[String], name: &str) -> Vec<String> {
    flag(args, name).map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default()
}

fn role(s: &str) -> Result<Role, String> {
    serde_json::from_value(serde_json::Value::String(s.into())).map_err(|_| format!("rôle inconnu : {s} (primaryLogo, horizontalLogo, verticalLogo, symbol, favicon, signature, secondaryLogo, other)"))
}

fn variant(s: &str) -> Result<Variant, String> {
    serde_json::from_value(serde_json::Value::String(s.into())).map_err(|_| format!("variante inconnue : {s} (color, black, white, monochrome, inverted, simplified)"))
}

fn run(args: &[String]) -> Result<(), String> {
    let usage = "usage : oracle-brand new|add-logo|add-color|add-font|status|plan|export|deliver DIR …";
    let cmd = args.first().ok_or(usage)?;
    let dir = PathBuf::from(args.get(1).ok_or(usage)?);
    match cmd.as_str() {
        "new" => {
            let name = args.get(2).ok_or("nom de marque manquant")?;
            let p = Project::create(&dir, name)?;
            println!("projet créé : {}", p.dir.join(oracle_brand::project::FILE_NAME).display());
        }
        "add-logo" => {
            let mut p = Project::open(&dir)?;
            let file = PathBuf::from(args.get(2).ok_or("fichier manquant")?);
            let file = std::path::absolute(&file).map_err(|e| e.to_string())?;
            if oracle_brand::render::master_kind(&file).is_none() {
                return Err(format!("{} : format non pris en charge", file.display()));
            }
            let id = p.data.new_id("asset");
            let r = role(&flag(args, "--role").unwrap_or_else(|| "other".into()))?;
            let v = variant(&flag(args, "--variant").unwrap_or_else(|| "color".into()))?;
            let label = flag(args, "--label").unwrap_or_else(|| r.label_fr().to_string());
            let source = p.store_path(&file);
            p.data.assets.push(Asset { id: id.clone(), role: r, label, variant: v, source, source_hash: String::new() });
            p.save()?;
            println!("{id}");
        }
        "add-color" => {
            let mut p = Project::open(&dir)?;
            let name = args.get(2).ok_or("nom manquant")?;
            let hex = args.get(3).ok_or("valeur #RRGGBB manquante")?;
            parse_hex(hex).ok_or_else(|| format!("{hex} : #RRGGBB attendu"))?;
            let id = p.data.new_id("color");
            p.data.colors.push(BrandColor { id: id.clone(), name: name.clone(), hex: hex.to_uppercase(), ..Default::default() });
            p.save()?;
            println!("{id}");
        }
        "add-font" => {
            let mut p = Project::open(&dir)?;
            let family = args.get(2).ok_or("famille manquante")?;
            let id = p.data.new_id("font");
            p.data.fonts.push(BrandFont { id: id.clone(), family: family.clone(), usage: args.get(3).cloned().unwrap_or_default(), ..Default::default() });
            p.save()?;
            println!("{id}");
        }
        "status" => {
            let p = Project::open(&dir)?;
            println!("{} ({}, {})", p.data.brand_name, p.data.version, p.data.status.label_fr());
            for a in &p.data.assets {
                println!("  {:<16} {:<20} {:<10} {:<32} {}", a.id, a.role.label_fr(), a.variant.label_fr(), p.source_state(a).label_fr(), a.source);
            }
            for d in &p.data.deliveries {
                println!("  livraison {:<8} {} fichier(s) {}", d.folder, d.outputs, if d.delivered { "livrée" } else { "brouillon" });
            }
        }
        "plan" => {
            let p = Project::open(&dir)?;
            let pl = export::plan(&p, &ids(args, "--assets"), &ids(args, "--presets"));
            println!("dossier : {}", p.deliveries_root().join(&pl.folder).display());
            for o in &pl.outputs {
                match &o.blocked {
                    None => println!("  {}", o.path),
                    Some(b) => println!("  {}  — BLOQUÉ : {b}", o.path),
                }
            }
        }
        "export" => {
            let mut p = Project::open(&dir)?;
            let mut assets = ids(args, "--assets");
            if args.iter().any(|a| a == "--changed") {
                assets = export::stale(&p).into_iter().filter(|(_, s)| *s == SourceState::Modified || *s == SourceState::New).map(|(id, _)| id).collect();
                if assets.is_empty() {
                    println!("aucune source modifiée");
                    return Ok(());
                }
            }
            let pl = export::plan(&p, &assets, &ids(args, "--presets"));
            let engines = Engines::discover();
            let report = export::run(&mut p, &engines, &pl, None, &AtomicBool::new(false), |done, total, path| eprintln!("[{done}/{total}] {path}"));
            p.save()?;
            println!("{} fichier(s) dans {}", report.succeeded(), report.folder.display());
            let failures: Vec<_> = report.failures().collect();
            for f in &failures {
                println!("  ÉCHEC {} — {}", f.path, f.error.as_deref().unwrap_or(""));
            }
            if !failures.is_empty() {
                return Err(format!("{} échec(s)", failures.len()));
            }
        }
        "deliver" => {
            let mut p = Project::open(&dir)?;
            let folder = args.get(2).ok_or("dossier de livraison manquant")?;
            let d = p.data.deliveries.iter_mut().find(|d| &d.folder == folder).ok_or_else(|| format!("pas de livraison {folder}"))?;
            d.delivered = true;
            p.save()?;
            println!("{folder} marquée livrée : elle ne sera plus réécrite");
        }
        _ => return Err(usage.into()),
    }
    Ok(())
}
