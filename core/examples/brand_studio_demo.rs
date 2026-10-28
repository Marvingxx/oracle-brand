//! Creates a new fictional identity and real offline exports. No existing project is changed.
use oracle_brand::{
    Project,
    guide::{self, Rights},
    model::{Asset, BrandColor, BrandFont, Role, Variant},
    project::hash_file,
    render::Engines,
    studio::{self, Concept, Decision, Tool},
};

fn main() -> Result<(), String> {
    let dir = std::env::args()
        .nth(1)
        .ok_or("Indiquer un NOUVEAU dossier de démonstration.")?;
    let mut p = Project::create(std::path::Path::new(&dir), "ORACLE — STUDIO FICTIF")?;
    p.data.client = "Démonstration uniquement ; aucun visa client".into();
    p.data.notes = "Recette locale de logiciels conçus indépendamment. Les figures sont des créations géométriques de démonstration, pas des ressources Akrivi.".into();
    for (id, path, content) in [
        (
            "anneau",
            "anneau.svg",
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="200" viewBox="0 0 400 200"><circle cx="130" cy="100" r="60" fill="none" stroke="#D31C3B" stroke-width="24"/><path d="M130 60v80M90 100h80" stroke="#1B1A17" stroke-width="12"/><path d="M230 55h110v22H230zM230 95h80v22h-80zM230 135h110v22H230z" fill="#1B1A17"/></svg>"##,
        ),
        (
            "signal",
            "signal.svg",
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="200" viewBox="0 0 400 200"><rect x="65" y="35" width="130" height="130" rx="24" fill="#1B1A17"/><path d="M95 125l35-70 35 70M110 100h40" stroke="#D31C3B" stroke-width="14" fill="none"/><path d="M235 55h100v22H235zM235 95h80v22h-80zM235 135h100v22H235z" fill="#1B1A17"/></svg>"##,
        ),
    ] {
        std::fs::write(p.dir.join(path), content).map_err(|e| e.to_string())?;
        let hash = hash_file(&p.dir.join(path))?;
        p.data.assets.push(Asset {
            id: id.into(),
            role: Role::PrimaryLogo,
            label: format!("Piste fictive {id}"),
            variant: Variant::Color,
            source: path.into(),
            source_hash: hash.clone(),
        });
        p.data.guide.rights.insert(
            id.into(),
            Rights {
                allow_delivery: true,
                evidence: "Création géométrique originale de cet exemple ; aucun fichier tiers."
                    .into(),
                reviewer: "Oracle — auteur de la démonstration, sans visa client".into(),
                reviewed_at: oracle_brand::now_rfc3339(),
                source_hash: hash,
            },
        );
    }
    for (id, name, hex) in [
        ("red", "Rouge", "#D31C3B"),
        ("ink", "Charbon", "#1B1A17"),
        ("paper", "Papier", "#F3EFE6"),
    ] {
        p.data.colors.push(BrandColor {
            id: id.into(),
            name: name.into(),
            hex: hex.into(),
            ..Default::default()
        });
    }
    for (id, family, usage) in [
        ("title", "Inter", "Titres · référence"),
        ("body", "Arial", "Texte · référence"),
    ] {
        p.data.fonts.push(BrandFont {
            id: id.into(),
            family: family.into(),
            usage: usage.into(),
            ..Default::default()
        });
    }
    p.data.studio.asset_id = "anneau".into();
    p.data.studio.headline = "Une identité, plusieurs pistes. Une bibliothèque partagée.".into();
    for (asset, title, rationale, decision) in [
        (
            "anneau",
            "01 · Le repère",
            "Un anneau ouvert sur un repère central. La piste évoque la navigation et le travail collectif. Hypothèse créative fictive, à discuter.",
            Decision::Proposed,
        ),
        (
            "signal",
            "02 · Le signal",
            "Une silhouette plus compacte, construite autour d'un signe directionnel. Le contraste vise la lisibilité à petite taille ; essais réels encore nécessaires.",
            Decision::Proposed,
        ),
    ] {
        let mut c = Concept::new(asset.into());
        c.title = title.into();
        c.rationale = rationale.into();
        c.decision = decision;
        p.data.studio.concepts.push(c);
    }
    p.data.guide.audience = "Démonstration de Studio Oracle, supports écran, français".into();
    p.save()?;
    let engines = Engines::discover();
    let mut folders = std::collections::BTreeMap::new();
    for (name, tool) in [
        ("grille", Tool::Grid),
        ("bento", Tool::Bento),
        ("presentations", Tool::Presentations),
    ] {
        folders.insert(
            name,
            studio::export(&p, &engines, tool)?.display().to_string(),
        );
    }
    folders.insert(
        "espaceClientBrouillon",
        guide::export_hub(&p, &engines, true)?.display().to_string(),
    );
    let summary = serde_json::json!({"fictional":true,"project":p.dir,"folders":folders,"clientApproval":false});
    std::fs::write(
        p.dir.join("DEMONSTRATION.json"),
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    Ok(())
}
