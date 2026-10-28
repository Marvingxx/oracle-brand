//! `oracle-brand`: the Brand Kit from the command line, for scripts and the reproducible demo.
//!
//! ```text
//! oracle-brand new DIR "Nom de marque"
//! oracle-brand add-logo DIR FILE --role primaryLogo [--variant color] [--label "…"]
//! oracle-brand add-color DIR "Nom" #RRGGBB
//! oracle-brand add-font DIR "Famille" "Usage"
//! oracle-brand status DIR
//! oracle-brand plan DIR [--assets id,id] [--presets id,id]
//! oracle-brand export DIR [--assets id,id] [--presets id,id] [--changed] [--mockups [--templates DIR]]
//! oracle-brand deliver DIR FOLDER
//! oracle-brand guide-check DIR
//! oracle-brand guide-export DIR [--draft]
//! oracle-brand studio-export DIR grid|bento|presentations
//! oracle-brand hub-export DIR [--draft]
//!
//! oracle-brand mockup-demo-photo OUT_DIR                      (photo et masque de synthèse)
//! oracle-brand template-new TEMPLATES_DIR PHOTO "Nom"
//! oracle-brand template-zone TEMPLATE ZONE --corners x,y;x,y;x,y;x,y [--ratio 85:55] [--name …] [--blend multiply] [--opacity 1]
//! oracle-brand template-occlusion TEMPLATE FILE
//! oracle-brand scene-new DIR TEMPLATE "Nom"
//! oracle-brand scene-fill DIR SCENE ZONE (--asset ID | --file PATH | --none) [--background COLOR_ID|#RRGGBB] [--fit contain|cover|stretch] [--scale 0.8] [--rotation 0] [--offset x,y]
//! oracle-brand scenes DIR [--templates DIR]
//! oracle-brand render DIR [--scenes a.json,b.json] [--outdated] [--templates DIR]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use oracle_brand::export;
use oracle_brand::mockup::{self, Content, Paint, Scene, SceneState, Template};
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
    let usage = "usage : oracle-brand new|add-logo|add-color|add-font|status|plan|export|deliver|guide-check|guide-export|studio-export|hub-export|vectorize|tokens|template-new|template-zone|template-occlusion|scene-new|scene-fill|scenes|render DIR …";
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
        "guide-check" => {
            let p = Project::open(&dir)?;
            let report = oracle_brand::guide::check(&p);
            println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
            if !report.complete() { return Err(format!("{} blocage(s) avant livraison", report.blockers.len())); }
        }
        "guide-export" => {
            let mut p = Project::open(&dir)?;
            let folder = oracle_brand::guide::export(&p, &Engines::discover(), args.iter().any(|s| s == "--draft"))?;
            let reference = p.store_path(&folder.join("guide.html"));
            p.data.guide_documents.push(reference);
            p.save().map_err(|e| format!("Guide créé dans {}, mais son lien n'a pas pu être enregistré : {e}", folder.display()))?;
            println!("{}", folder.display());
        }
        "studio-export" => {
            use oracle_brand::studio::{self, Tool};
            let p = Project::open(&dir)?;
            let tool = match args.get(2).map(String::as_str) {
                Some("grid") => Tool::Grid, Some("bento") => Tool::Bento, Some("presentations") => Tool::Presentations,
                _ => return Err("Indiquer grid, bento ou presentations.".into()),
            };
            println!("{}", studio::export(&p, &Engines::discover(), tool)?.display());
        }
        "hub-export" => {
            let p = Project::open(&dir)?;
            println!("{}", oracle_brand::guide::export_hub(&p, &Engines::discover(), args.iter().any(|s| s == "--draft"))?.display());
        }
        "vectorize" => {
            // oracle-brand vectorize DIR ASSET_ID [--detailed]
            let mut p = Project::open(&dir)?;
            let asset = args.get(2).ok_or("identifiant du logo manquant (oracle-brand status les liste)")?;
            let detail = if args.iter().any(|a| a == "--detailed") { oracle_brand::vectorize::Detail::Detailed } else { oracle_brand::vectorize::Detail::Logo };
            let id = oracle_brand::vectorize::vectorize(&mut p, asset, detail)?;
            p.save()?;
            let a = p.data.assets.iter().find(|a| a.id == id).ok_or("logo créé introuvable")?;
            println!("{id} {}", a.source);
        }
        "tokens" => {
            // oracle-brand tokens DIR [-o FILE]: the W3C design tokens, to stdout or a file.
            let p = Project::open(&dir)?;
            let json = serde_json::to_string_pretty(&oracle_brand::tokens::tokens(&p.data)).map_err(|e| e.to_string())?;
            match flag(args, "-o") {
                Some(out) => std::fs::write(&out, json).map_err(|e| format!("{out} : {e}"))?,
                None => println!("{json}"),
            }
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
            let mut pl = export::plan(&p, &assets, &ids(args, "--presets"));
            if args.iter().any(|a| a == "--mockups") {
                let roots: Vec<PathBuf> = flag(args, "--templates").map(PathBuf::from).into_iter().collect();
                export::plan_mockups(&p, &mut pl, &roots);
            }
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
        "mockup-demo-photo" => {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let (photo, pen) = oracle_brand::mockup_demo::business_card();
            photo.save(dir.join("carte-oblique.png")).map_err(|e| e.to_string())?;
            pen.save(dir.join("stylo-premier-plan.png")).map_err(|e| e.to_string())?;
            let c = oracle_brand::mockup_demo::CARD;
            println!("{}", c.iter().map(|[x, y]| format!("{x},{y}")).collect::<Vec<_>>().join(";"));
        }
        "template-new" => {
            let photo = PathBuf::from(args.get(2).ok_or("photo manquante")?);
            let name = args.get(3).cloned().unwrap_or_default();
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            println!("{}", mockup::create_template(&dir, &photo, &name)?.display());
        }
        "template-zone" => {
            let (file, mut t) = Template::load(&dir)?;
            let zone_id = args.get(2).ok_or("zone manquante")?.clone();
            if !t.zones.iter().any(|z| z.id == zone_id) {
                t.zones.push(mockup::Zone::new(&zone_id, &zone_id));
            }
            let z = t.zones.iter_mut().find(|z| z.id == zone_id).ok_or("zone introuvable")?;
            if let Some(c) = flag(args, "--corners") {
                let pts: Vec<f64> = c.split([';', ',']).map(|v| v.trim().parse::<f64>().map_err(|_| format!("coordonnée invalide : {v}"))).collect::<Result<_, _>>()?;
                if pts.len() != 8 {
                    return Err("--corners attend 4 points x,y séparés par ;".into());
                }
                z.corners = [[pts[0], pts[1]], [pts[2], pts[3]], [pts[4], pts[5]], [pts[6], pts[7]]];
                oracle_brand::composite::zone_homography(&z.corners, 1.0)?;
            }
            if let Some(r) = flag(args, "--ratio") {
                let (w, h) = r.split_once(':').ok_or("--ratio attend L:H")?;
                z.ratio = [w.trim().parse().map_err(|_| "ratio invalide")?, h.trim().parse().map_err(|_| "ratio invalide")?];
            }
            if let Some(n) = flag(args, "--name") {
                z.name = n;
            }
            if let Some(b) = flag(args, "--blend") {
                z.blend = serde_json::from_value(serde_json::Value::String(b.clone())).map_err(|_| format!("mode inconnu : {b} (normal, multiply, screen)"))?;
            }
            if let Some(o) = flag(args, "--opacity") {
                z.opacity = o.parse().map_err(|_| "opacité invalide")?;
            }
            t.save(file.parent().ok_or("dossier du modèle")?)?;
            println!("{zone_id}");
        }
        "template-occlusion" => {
            let (file, mut t) = Template::load(&dir)?;
            let tdir = file.parent().ok_or("dossier du modèle")?;
            let mask = PathBuf::from(args.get(2).ok_or("masque manquant")?);
            t.occlusion = Some(mockup::adopt_file(tdir, &mask, "premier-plan")?);
            t.save(tdir)?;
        }
        "scene-new" => {
            let mut p = Project::open(&dir)?;
            let template = PathBuf::from(args.get(2).ok_or("modèle manquant")?);
            let template = std::path::absolute(&template).map_err(|e| e.to_string())?;
            println!("{}", mockup::create_scene(&mut p, &template, args.get(3).map(String::as_str).unwrap_or(""))?.display());
        }
        "scene-fill" => {
            let p = Project::open(&dir)?;
            let scene_file = PathBuf::from(args.get(2).ok_or("scène manquante")?);
            let zone = args.get(3).ok_or("zone manquante")?;
            let mut scene = Scene::load(&scene_file)?;
            let fill = scene.fill_mut(zone);
            if let Some(id) = flag(args, "--asset") {
                let a = p.data.assets.iter().find(|a| a.id == id).ok_or_else(|| format!("pas de logo {id}"))?;
                fill.content = Content::BrandAsset { asset_id: a.id.clone(), role: a.role, variant: a.variant };
            } else if let Some(f) = flag(args, "--file") {
                fill.content = Content::File { path: p.store_path(&std::path::absolute(PathBuf::from(f)).map_err(|e| e.to_string())?) };
            } else if args.iter().any(|a| a == "--none") {
                fill.content = Content::None;
            }
            if let Some(b) = flag(args, "--background") {
                fill.background = if b.starts_with('#') {
                    parse_hex(&b).ok_or_else(|| format!("{b} : #RRGGBB attendu"))?;
                    Some(Paint::Hex { hex: b.to_uppercase() })
                } else {
                    let index = p.data.colors.iter().position(|c| c.id == b).ok_or_else(|| format!("pas de couleur {b}"))?;
                    Some(Paint::BrandColor { color_id: b, index })
                };
            }
            if let Some(f) = flag(args, "--fit") {
                fill.fit = serde_json::from_value(serde_json::Value::String(f.clone())).map_err(|_| format!("ajustement inconnu : {f} (contain, cover, stretch)"))?;
            }
            if let Some(v) = flag(args, "--scale") {
                fill.scale = v.parse().map_err(|_| "échelle invalide")?;
            }
            if let Some(v) = flag(args, "--rotation") {
                fill.rotation = v.parse().map_err(|_| "rotation invalide")?;
            }
            if let Some(v) = flag(args, "--offset") {
                let (x, y) = v.split_once(',').ok_or("--offset attend x,y")?;
                fill.offset = [x.trim().parse().map_err(|_| "décalage invalide")?, y.trim().parse().map_err(|_| "décalage invalide")?];
            }
            scene.save(&scene_file)?;
        }
        "scenes" => {
            let p = Project::open(&dir)?;
            let roots: Vec<PathBuf> = flag(args, "--templates").map(PathBuf::from).into_iter().collect();
            for (stored, _, s) in mockup::scenes(&p) {
                match s {
                    Ok(s) => println!("  {:<28} {}", s.name, mockup::scene_state(&p, &s, &roots).label_fr()),
                    Err(e) => println!("  {stored:<28} {e}"),
                }
            }
        }
        "render" => {
            let p = Project::open(&dir)?;
            let roots: Vec<PathBuf> = flag(args, "--templates").map(PathBuf::from).into_iter().collect();
            let only = ids(args, "--scenes");
            let outdated = args.iter().any(|a| a == "--outdated");
            let engines = Engines::discover();
            let mut failures = 0;
            for (stored, file, s) in mockup::scenes(&p) {
                let name = file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if !only.is_empty() && !only.iter().any(|o| o == &name || o == &stored) {
                    continue;
                }
                if outdated && s.as_ref().is_ok_and(|s| mockup::scene_state(&p, s, &roots) == SceneState::UpToDate) {
                    continue;
                }
                match mockup::export_scene(&p, &file, &engines, &roots) {
                    Ok(out) => println!("  {}", out.display()),
                    Err(e) => {
                        failures += 1;
                        println!("  ÉCHEC {name} — {e}");
                    }
                }
            }
            if failures > 0 {
                return Err(format!("{failures} échec(s)"));
            }
        }
        _ => return Err(usage.into()),
    }
    Ok(())
}
