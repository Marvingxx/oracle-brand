//! Creates a NEW fictional project only. No client approval is fabricated.
use oracle_brand::Project;
use oracle_brand::guide::{self, Chapter, ContrastPair, ReviewState, Rights};
use oracle_brand::model::{Asset, BrandColor, BrandFont, ForbiddenUse, Role, Variant};
use oracle_brand::project::hash_file;
use oracle_brand::render::Engines;

fn main() -> Result<(), String> {
    let dir = std::env::args()
        .nth(1)
        .ok_or("Indiquer un NOUVEAU dossier de démonstration.")?;
    let mut p = Project::create(std::path::Path::new(&dir), "DÉMONSTRATION FICTIVE — Oracle")?;
    p.data.client = "Exemple pédagogique ; aucune marque client".into();
    p.data.notes = "Exemple généré pour vérifier le parcours Brand Guide. Tous les chapitres restent à relire et à valider par une personne. Aucun visa client.".into();
    let source = p.dir.join("symbole-demo.svg");
    std::fs::write(&source, r##"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="160" viewBox="0 0 320 160"><rect width="320" height="160" rx="20" fill="#1B1A17"/><circle cx="160" cy="80" r="48" fill="none" stroke="#D31C3B" stroke-width="20"/><path d="M160 42v76M122 80h76" stroke="#FFFFFF" stroke-width="8"/></svg>"##).map_err(|e| e.to_string())?;
    p.data.assets.push(Asset {
        id: "demo-logo".into(),
        role: Role::PrimaryLogo,
        label: "Symbole fictif de démonstration".into(),
        variant: Variant::Color,
        source: "symbole-demo.svg".into(),
        source_hash: hash_file(&source)?,
    });
    for (id, name, hex) in [
        ("demo-ink", "Charbon chaud", "#1B1A17"),
        ("demo-paper", "Blanc", "#FFFFFF"),
        ("demo-red", "Rouge Oracle", "#D31C3B"),
    ] {
        p.data.colors.push(BrandColor {
            id: id.into(),
            name: name.into(),
            hex: hex.into(),
            notes: "Exemple écran uniquement. Aucune valeur d'impression déduite.".into(),
            ..Default::default()
        });
    }
    p.data.fonts.push(BrandFont {
        id: "demo-font".into(),
        family: "Inter (référence, fichier non fourni)".into(),
        usage: "Titres et texte ; système de repli sans-serif".into(),
        weights: vec!["Regular 400".into(), "Semibold 600".into()],
        license_url: "https://github.com/rsms/inter/blob/master/LICENSE.txt".into(),
        local_files: Vec::new(),
        redistributable: false,
    });
    p.data.rules.clear_space =
        "Conserver au moins une épaisseur d'anneau autour du symbole.".into();
    p.data.rules.min_size =
        "Exemple : largeur minimale 64 px ; à confirmer sur les supports réels.".into();
    p.data.rules.allowed_backgrounds =
        "Fond blanc ou charbon ; le fichier de démonstration contient son propre fond.".into();
    for text in [
        "Ne pas étirer le symbole.",
        "Ne pas recolorer les éléments sans validation.",
        "Ne pas employer cet exemple comme identité client approuvée.",
    ] {
        p.data.rules.forbidden.push(ForbiddenUse {
            text: text.into(),
            example: String::new(),
        });
    }
    p.data.guide.audience =
        "Designer et destinataire d'une démonstration locale ; supports numériques, français."
            .into();
    p.data.guide.owner =
        "Démonstration Oracle — responsable réel à renseigner avant livraison".into();
    p.data.guide.rights.insert("demo-logo".into(), Rights { allow_delivery: true, evidence: "SVG géométrique original créé automatiquement pour cet exemple ; aucune ressource Akrivi ou client utilisée.".into(), reviewer: "Oracle — création de la ressource de démonstration, sans visa client".into(), reviewed_at: oracle_brand::now_rfc3339(), source_hash: hash_file(&source)? });
    for (foreground, background, context) in [
        ("demo-ink", "demo-paper", "Texte courant sur blanc"),
        ("demo-paper", "demo-red", "Texte courant sur rouge"),
    ] {
        p.data.guide.contrast_pairs.push(ContrastPair {
            foreground_id: foreground.into(),
            background_id: background.into(),
            context: context.into(),
            large_text_or_graphic: false,
        });
    }
    let examples = [
        (
            "BG-01",
            "Démonstration locale en français, pour écran. Livrer le guide HTML, la fiche de référence et le symbole SVG. Impression, traductions et modèles commerciaux exclus. Responsable et visa réels à définir.",
        ),
        (
            "BG-02",
            "Projet fictif : aider une équipe créative à retrouver ses ressources. Promesse de clarté, personnalité directe et méthodique. Ces hypothèses illustrent les champs ; une stratégie client nécessite un entretien et des preuves.",
        ),
        (
            "BG-03",
            "Symbole principal fourni dans les ressources liées. Préserver ses proportions, sa zone de protection et sa lisibilité. Les autres variantes et les essais de petite taille restent à produire et à approuver.",
        ),
        (
            "BG-04",
            "Le charbon sert de fond, le blanc de texte, le rouge d'accent. Les valeurs écran et les associations contrôlées figurent dans la palette partagée. Aucune équivalence CMJN, Pantone ou RAL n'est inventée.",
        ),
        (
            "BG-05",
            "Exemple Inter : titres 32 px semibold, texte 16 px regular, interligne 1,5. Repli sans-serif. Tester accents, chiffres et caractères requis. Consulter la licence liée ; aucun fichier de police n'est distribué.",
        ),
        (
            "BG-06",
            "Univers illustratif sobre et géométrique. Le symbole joint est original et sa provenance est renseignée. Aucune photographie ni banque d'icônes n'est livrée ; leurs cadrages, traitements et droits sont à préciser si ce périmètre est étendu.",
        ),
        (
            "BG-07",
            "Exemple écran : marges de 24 px, espacements par multiples de 8 px, titre puis contenu puis action. Ces décisions doivent être éprouvées sur mobile et sur écran large. Aucun gabarit natif n'est inclus dans cette démonstration.",
        ),
        (
            "BG-08",
            "Ton clair, direct, utile ; vocabulaire concret. Préférer « Ouvrir la bibliothèque » à « Exploitez vos ressources ». Expliquer une erreur et son action corrective. Traductions et messages sensibles nécessitent une relecture humaine.",
        ),
        (
            "BG-09",
            "Les deux associations définies sont contrôlées en sRGB opaque. Conserver aussi des libellés et une hiérarchie lisible. Vérifier au clavier, au zoom et avec les alternatives textuelles ; ces essais manuels ne sont pas certifiés par ce fichier.",
        ),
        (
            "BG-10",
            "Supports de démonstration : le présent guide et sa fiche courte. Aucun modèle social, document bureautique ni interface client n'est prétendu approuvé. Joindre ensuite les ressources exactes et leurs limites de modification.",
        ),
        (
            "BG-11",
            "Livraison locale HTML, SVG et aperçu PNG, avec manifeste et empreintes. Sources préservées. Fonts non incluses. Impression hors périmètre ; définir les profils et effectuer un bon à tirer avant tout futur usage imprimé.",
        ),
        (
            "BG-12",
            "Version v1 de démonstration. Le responsable réel et le circuit de visa restent à désigner. Toute modification invalide les validations précédentes. Conserver les livraisons datées et communiquer les retraits de ressources aux destinataires.",
        ),
    ];
    for spec in guide::CHAPTERS.iter().filter(|s| !s.optional) {
        let text = examples
            .iter()
            .find(|(id, _)| *id == spec.id)
            .map(|(_, text)| *text)
            .ok_or("Chapitre de démonstration absent")?;
        let mut chapter = Chapter {
            state: ReviewState::Draft,
            ..Default::default()
        };
        for field in spec.fields {
            chapter.fields.insert(
                (*field).into(),
                format!("EXEMPLE À ADAPTER — {field}\n{text}"),
            );
        }
        if spec.id == "BG-03" {
            chapter.asset_ids.push("demo-logo".into());
        }
        p.data.guide.chapters.insert(spec.id.into(), chapter);
    }
    p.save()?;
    let delivery = guide::export(&p, &Engines::discover(), true)?;
    let path = delivery.join("guide.html");
    p.data.guide_documents.push(p.store_path(&path));
    p.save()?;
    println!(
        "Projet fictif : {}\nBrouillon : {}",
        p.dir.display(),
        path.display()
    );
    Ok(())
}
