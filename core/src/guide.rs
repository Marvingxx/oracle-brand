//! Brand guide source, human approvals, deterministic checks and offline delivery.
//! Shared identity records remain the only source for logos, colours, typography and rules.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::model::{Role, parse_hex};
use crate::project::hash_file;
use crate::{BrandProject, Project};

#[derive(Clone, Copy)]
pub struct ChapterSpec {
    pub id: &'static str,
    pub title: &'static str,
    /// The designer supplies each of these decisions; Oracle does not invent them.
    pub fields: &'static [&'static str],
    pub optional: bool,
}

pub const CHAPTERS: &[ChapterSpec] = &[
    ChapterSpec {
        id: "BG-01",
        title: "Périmètre et gouvernance",
        fields: &[
            "Destinataires et supports",
            "Pays et langues",
            "Propriétaire et contact",
            "Livrables et exclusions",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-02",
        title: "Plateforme de marque",
        fields: &[
            "Mission et vision",
            "Public et besoins",
            "Positionnement et promesse",
            "Valeurs, preuves et personnalité",
            "Messages clés",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-03",
        title: "Logos",
        fields: &[
            "Rôles et variantes requis",
            "Construction et unités de protection",
            "Tailles et contextes",
            "Exemples corrects et interdits",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-04",
        title: "Couleurs",
        fields: &[
            "Rôles et proportions",
            "Associations et usages",
            "Écran et impression",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-05",
        title: "Typographies",
        fields: &[
            "Hiérarchie et tailles",
            "Graisses, interlignage et approches",
            "Replis et caractères requis",
            "Licences et acquisition",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-06",
        title: "Univers visuel",
        fields: &[
            "Photographie et cadrages",
            "Illustrations, icônes et motifs",
            "Traitements et contre-exemples",
            "Sources et droits",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-07",
        title: "Composition",
        fields: &[
            "Grilles, marges et espacements",
            "Hiérarchie et formats",
            "Exemples de mise en page",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-08",
        title: "Voix et rédaction",
        fields: &[
            "Voix et variations du ton",
            "Vocabulaire et règles éditoriales",
            "Exemples avant et après",
            "Messages sensibles et traductions",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-09",
        title: "Accessibilité",
        fields: &[
            "Contextes des associations de couleurs",
            "Lisibilité, alternatives et information sans couleur",
            "Vérifications manuelles et réserves",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-10",
        title: "Applications et modèles",
        fields: &[
            "Supports prioritaires et dimensions",
            "Exemples approuvés et ressources liées",
            "Consignes de modification et limites",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-11",
        title: "Production et livraison",
        fields: &[
            "Formats et logiciels de destination",
            "Écran, impression et profils",
            "Nommage, arborescence et contrôle final",
            "Droits et exclusions de fichiers",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "BG-12",
        title: "Maintenance",
        fields: &[
            "Responsable et circuit de validation",
            "Version de référence et changements",
            "Révision, archivage et anciens assets",
        ],
        optional: false,
    },
    ChapterSpec {
        id: "architecture",
        title: "Architecture et co-branding",
        fields: &[
            "Niveaux et relations",
            "Associations et règles partenaires",
            "Exemples et exceptions",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "digital",
        title: "Système numérique",
        fields: &[
            "Tokens et composants",
            "États et interactions",
            "Responsive et accessibilité",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "motion",
        title: "Motion",
        fields: &[
            "Mouvement, durées et courbes",
            "Transitions, logo et exemples",
            "Réduction des animations",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "audio",
        title: "Identité sonore",
        fields: &[
            "Signature et usages",
            "Mixage et alternatives",
            "Fichiers et droits",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "packaging",
        title: "Packaging et signalétique",
        fields: &[
            "Gabarits, matériaux et dimensions",
            "Production et contrôles métier",
            "Exemples et mentions",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "data",
        title: "Données et graphiques",
        fields: &[
            "Graphiques, axes et unités",
            "Palettes et alternatives accessibles",
            "Exemples",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "i18n",
        title: "Internationalisation",
        fields: &[
            "Langues et écritures",
            "Adaptations typographiques et culturelles",
            "Traduction et validation locale",
        ],
        optional: true,
    },
    ChapterSpec {
        id: "mascot",
        title: "Mascotte",
        fields: &[
            "Construction et proportions",
            "Expressions, poses et usages",
            "Interdits et fichiers",
        ],
        optional: true,
    },
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReviewState {
    #[default]
    Missing,
    Draft,
    Validated,
    NotApplicable,
}
impl ReviewState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Missing => "Manquant",
            Self::Draft => "Brouillon",
            Self::Validated => "Validé",
            Self::NotApplicable => "Non applicable",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Chapter {
    pub fields: BTreeMap<String, String>,
    pub asset_ids: Vec<String>,
    pub state: ReviewState,
    pub reviewer: String,
    pub reviewed_at: String,
    pub reason: String,
    pub approved_fingerprint: String,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Rights {
    pub allow_delivery: bool,
    pub evidence: String,
    pub reviewer: String,
    pub reviewed_at: String,
    /// Permission applies to the reviewed file, not an unrelated replacement at the same path.
    pub source_hash: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ContrastPair {
    pub foreground_id: String,
    pub background_id: String,
    pub context: String,
    /// Normal text: 4.5; large text or meaningful graphics: 3. Context is human-reviewed.
    pub large_text_or_graphic: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Guide {
    pub audience: String,
    pub owner: String,
    pub reviewer: String,
    pub print_required: bool,
    /// Keys of optional chapters. Unknown keys are preserved and block final delivery.
    pub modules: Vec<String>,
    pub chapters: BTreeMap<String, Chapter>,
    pub rights: BTreeMap<String, Rights>,
    pub contrast_pairs: Vec<ContrastPair>,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

pub fn applicable<'a>(guide: &'a Guide) -> impl Iterator<Item = &'static ChapterSpec> + 'a {
    CHAPTERS
        .iter()
        .filter(|s| !s.optional || guide.modules.iter().any(|id| id == s.id))
}

/// Human approvals are invalidated by edits to shared identity, scope or guide content.
/// Delivery history and approval metadata do not themselves invalidate content.
pub fn fingerprint(data: &BrandProject) -> String {
    let content: BTreeMap<_, _> = data
        .guide
        .chapters
        .iter()
        .map(|(id, c)| (id, (&c.fields, &c.asset_ids, &c.reason, &c.extensions)))
        .collect();
    let value = serde_json::json!({ "name": data.brand_name, "client": data.client, "version": data.version,
        "assets": data.assets, "colors": data.colors, "fonts": data.fonts, "rules": data.rules,
        "audience": data.guide.audience, "owner": data.guide.owner, "print": data.guide.print_required,
        "modules": data.guide.modules, "content": content, "rights": data.guide.rights,
        "contrasts": data.guide.contrast_pairs, "extensions": data.guide.extensions });
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}

pub fn approve(data: &mut BrandProject, id: &str, not_applicable: bool) -> Result<(), String> {
    let spec = CHAPTERS
        .iter()
        .find(|s| s.id == id)
        .ok_or("chapitre inconnu")?;
    if data.guide.reviewer.trim().is_empty() {
        return Err("Indique la personne qui valide.".into());
    }
    let chapter = data
        .guide
        .chapters
        .get(id)
        .ok_or("Renseigne d'abord le chapitre.")?;
    if not_applicable {
        if !spec.optional || chapter.reason.trim().is_empty() {
            return Err("Seul un module optionnel peut être écarté, avec justification.".into());
        }
    } else if spec
        .fields
        .iter()
        .any(|f| chapter.fields.get(*f).is_none_or(|v| v.trim().is_empty()))
    {
        return Err("Renseigne chaque décision du chapitre avant de le valider.".into());
    }
    let hash = fingerprint(data);
    let reviewer = data.guide.reviewer.clone();
    if let Some(chapter) = data.guide.chapters.get_mut(id) {
        chapter.state = if not_applicable {
            ReviewState::NotApplicable
        } else {
            ReviewState::Validated
        };
        chapter.reviewer = reviewer;
        chapter.reviewed_at = crate::now_rfc3339();
        chapter.approved_fingerprint = hash;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
pub struct Issue {
    pub code: String,
    pub message: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub applicable_chapters: usize,
    pub approved_chapters: usize,
    pub blockers: Vec<Issue>,
    pub warnings: Vec<Issue>,
}
impl CheckReport {
    pub fn complete(&self) -> bool {
        self.blockers.is_empty()
    }
}

pub fn contrast(foreground: &str, background: &str) -> Option<f64> {
    fn luminance(hex: &str) -> Option<f64> {
        let [r, g, b] = parse_hex(hex)?;
        let linear = |v: u8| {
            let x = f64::from(v) / 255.0;
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        Some(0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b))
    }
    let a = luminance(foreground)?;
    let b = luminance(background)?;
    Some((a.max(b) + 0.05) / (a.min(b) + 0.05))
}

pub fn authorized(project: &Project, id: &str) -> bool {
    let Some(a) = project.data.assets.iter().find(|a| a.id == id) else {
        return false;
    };
    let Some(r) = project.data.guide.rights.get(id) else {
        return false;
    };
    r.allow_delivery
        && !r.evidence.trim().is_empty()
        && !r.reviewer.trim().is_empty()
        && !r.reviewed_at.is_empty()
        && !r.source_hash.is_empty()
        && hash_file(&project.resolve(&a.source)).is_ok_and(|h| h == r.source_hash)
}

pub fn check(project: &Project) -> CheckReport {
    let data = &project.data;
    let g = &data.guide;
    let mut report = CheckReport {
        applicable_chapters: applicable(g).count(),
        approved_chapters: 0,
        blockers: vec![],
        warnings: vec![],
    };
    let mut block = |code: &str, message: String| {
        report.blockers.push(Issue {
            code: code.into(),
            message,
        })
    };
    if !data.duplicate_ids().is_empty()
        || data.assets.iter().any(|a| a.id.trim().is_empty())
        || data.colors.iter().any(|c| c.id.trim().is_empty())
        || data.fonts.iter().any(|f| f.id.trim().is_empty())
    {
        block(
            "ids",
            "Les identifiants des ressources doivent être présents et uniques.".into(),
        );
    }
    if data.brand_name.trim().is_empty()
        || g.audience.trim().is_empty()
        || g.owner.trim().is_empty()
    {
        block(
            "scope",
            "Nom de marque, destinataires et responsable sont requis.".into(),
        );
    }
    let hash = fingerprint(data);
    for spec in applicable(g) {
        let Some(c) = g.chapters.get(spec.id) else {
            block(spec.id, format!("{} : contenu manquant", spec.title));
            continue;
        };
        let approved = match c.state {
            ReviewState::Validated => spec
                .fields
                .iter()
                .all(|f| c.fields.get(*f).is_some_and(|s| !s.trim().is_empty())),
            ReviewState::NotApplicable => spec.optional && !c.reason.trim().is_empty(),
            _ => false,
        };
        if !approved || c.reviewer.trim().is_empty() || c.reviewed_at.is_empty() {
            block(
                spec.id,
                format!("{} : décisions ou validation manquantes", spec.title),
            );
        } else if c.approved_fingerprint != hash {
            block(
                spec.id,
                format!(
                    "{} : validation obsolète après modification de l'identité ou du guide",
                    spec.title
                ),
            );
        } else {
            report.approved_chapters += 1;
        }
        for id in &c.asset_ids {
            if !data.assets.iter().any(|a| &a.id == id) {
                block(
                    spec.id,
                    format!("{} : référence d'asset supprimée ({id})", spec.title),
                );
            }
        }
    }
    for module in &g.modules {
        if !CHAPTERS.iter().any(|s| s.optional && s.id == module) {
            block("module", format!("Module inconnu : {module}"));
        }
    }
    if !data.assets.iter().any(|a| a.role == Role::PrimaryLogo) {
        block("logos", "Logo principal requis.".into());
    }
    for a in &data.assets {
        if !authorized(project, &a.id) {
            block(
                "rights",
                format!(
                    "{} : fichier absent, modifié ou autorisation de remise non établie",
                    a.label
                ),
            );
        }
        let ext = Path::new(&a.source)
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !["svg", "png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
            block(
                "format",
                format!(
                    "{} : export de guide limité aux maîtres SVG, PNG, JPEG et WebP ; préparer une variante autorisée pour ce format",
                    a.label
                ),
            );
        }
    }
    if data.colors.is_empty() {
        block("colors", "Palette requise.".into());
    }
    for c in &data.colors {
        if parse_hex(&c.hex).is_none() {
            block("colors", format!("{} : HEX invalide", c.name));
        }
        if g.print_required
            && c.cmyk.as_ref().is_none_or(|v| {
                v.profile.trim().is_empty()
                    || [v.c, v.m, v.y, v.k]
                        .iter()
                        .any(|n| !n.is_finite() || !(0.0..=100.0).contains(n))
            })
        {
            block(
                "print",
                format!(
                    "{} : CMYK explicite (0–100) et profil requis pour l'impression",
                    c.name
                ),
            );
        }
    }
    if data.fonts.is_empty() {
        block(
            "fonts",
            "Au moins une typographie et sa licence sont requises.".into(),
        );
    }
    for f in &data.fonts {
        if f.family.trim().is_empty()
            || f.usage.trim().is_empty()
            || f.weights.is_empty()
            || f.weights.iter().any(|w| w.trim().is_empty())
            || !safe_url(&f.license_url)
        {
            block(
                "fonts",
                format!(
                    "{} : famille, usage, graisses ou lien de licence manquant",
                    f.family
                ),
            );
        }
    }
    if data.rules.clear_space.trim().is_empty()
        || data.rules.min_size.trim().is_empty()
        || data.rules.allowed_backgrounds.trim().is_empty()
        || data.rules.forbidden.is_empty()
        || data
            .rules
            .forbidden
            .iter()
            .any(|r| r.text.trim().is_empty())
    {
        block(
            "rules",
            "Renseigner protection, taille minimale, fonds et au moins un interdit.".into(),
        );
    }
    if g.contrast_pairs.is_empty() {
        block(
            "contrast",
            "Définir au moins une association de couleurs et son contexte.".into(),
        );
    }
    for pair in &g.contrast_pairs {
        let fg = data.colors.iter().find(|c| c.id == pair.foreground_id);
        let bg = data.colors.iter().find(|c| c.id == pair.background_id);
        let ratio = fg.zip(bg).and_then(|(a, b)| contrast(&a.hex, &b.hex));
        let threshold = if pair.large_text_or_graphic { 3.0 } else { 4.5 };
        if pair.context.trim().is_empty() || ratio.is_none_or(|r| r < threshold) {
            block(
                "contrast",
                format!(
                    "{} : association absente, contexte manquant ou contraste inférieur à {threshold}:1",
                    pair.context
                ),
            );
        }
    }
    report.warnings.push(Issue { code: "manual-review".into(), message: "Les contrastes portent sur des couleurs opaques unies. Typographie réelle, images, transparences, pertinence des contenus et conditions de licence nécessitent une revue humaine. Aucune certification automatique d'accessibilité.".into() });
    report
}

fn safe_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && !url.chars().any(char::is_control)
        && url
            .split("://")
            .nth(1)
            .is_some_and(|v| !v.is_empty() && !v.starts_with('/'))
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn para(s: &str) -> String {
    format!("<p class=prose>{}</p>", esc(s))
}

const CSS: &str = "*{box-sizing:border-box}body{margin:0;background:#1B1A17;color:#f4f0e8;font:16px/1.65 system-ui,sans-serif}a{color:#ff9baa}header,main,footer{max-width:1120px;margin:auto;padding:40px}header{padding-top:72px;border-bottom:3px solid #D31C3B}h1{font-size:clamp(2rem,5vw,4rem);line-height:1.1}h2{margin:0 0 20px;font-size:1.8rem}h3{font-size:1rem;color:#f4d2d7}.eyebrow{color:#ff9baa;letter-spacing:.15em;text-transform:uppercase}section{background:#25231f;border:1px solid #484139;border-radius:12px;padding:30px;margin:24px 0;break-inside:avoid}.prose{white-space:pre-wrap;overflow-wrap:anywhere}nav{display:flex;flex-wrap:wrap;gap:8px 20px}table{border-collapse:collapse;width:100%;margin:20px 0}td,th{text-align:left;padding:12px;border-bottom:1px solid #62594e;overflow-wrap:anywhere}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(230px,1fr));gap:16px}.asset{padding:20px;background:#fff;color:#1B1A17;border-radius:8px}.asset a{color:#9e1630}.asset img{max-width:100%;height:160px;object-fit:contain}.swatch{display:block;min-height:72px;border:1px solid #888;border-radius:8px}.badge{display:inline-block;border:1px solid #a3907c;border-radius:30px;padding:3px 12px}.draft{background:#572b16;color:#fff;padding:20px}small{color:#cfc4b6}.asset small{color:#494238}@media(max-width:600px){header,main,footer{padding:20px}section{padding:18px}table{font-size:13px}}@media print{body{background:white;color:#171717;font-size:11pt}header,main,footer{padding:14px;max-width:none}section{background:white;border-color:#aaa;break-inside:auto}h3,small{color:#333}a{color:#8b1530}nav,.print-hide{display:none}.draft{border:3px solid #a00;background:white;color:#a00}.asset img{height:100px}h2,h3{break-after:avoid}}";

fn html(
    project: &Project,
    report: &CheckReport,
    draft: bool,
    resources: &BTreeMap<String, (String, String)>,
    quick: bool,
) -> String {
    let d = &project.data;
    let mut out = format!(
        "<!doctype html><html lang=fr><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><meta http-equiv=Content-Security-Policy content=\"default-src 'none'; img-src 'self' data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'\"><title>{} — {}</title><style>{CSS}</style><header><div class=eyebrow>Oracle · {}</div><h1>{}</h1><p>{} · {}</p><span class=badge>{}</span></header><main>",
        esc(&d.brand_name),
        if quick {
            "Fiche de référence"
        } else {
            "Brand guide"
        },
        if quick {
            "Référence rapide"
        } else {
            "Brand guide"
        },
        esc(&d.brand_name),
        esc(&d.version),
        esc(&d.guide.audience),
        if draft {
            "BROUILLON — non livrable comme guide validé"
        } else {
            "Validé pour le périmètre déclaré"
        }
    );
    if draft {
        out.push_str(&format!("<div class=draft>{} blocage(s) à résoudre. Les fichiers sans autorisation valide sont exclus.</div>", report.blockers.len()));
    }
    out.push_str("<p><a href=guide.html>Guide complet</a> · <a href=fiche-reference.html>Fiche de référence</a> · <a href=controle.json>Rapport de contrôle</a> · <a href=manifest.json>Manifeste</a></p>");
    if !quick {
        out.push_str("<nav aria-label=Sommaire>");
        for s in applicable(&d.guide) {
            out.push_str(&format!(
                "<a href='#{}'>{} · {}</a>",
                s.id,
                s.id,
                esc(s.title)
            ));
        }
        out.push_str("</nav>");
    }
    out.push_str("<section><h2>Ressources de l'identité</h2><div class=grid>");
    for a in &d.assets {
        out.push_str(&format!(
            "<div class=asset><h3 style='color:#1B1A17'>{}</h3>",
            esc(&a.label)
        ));
        if let Some((original, preview)) = resources.get(&a.id) {
            out.push_str(&format!("<img src='{}' alt='{}'><p><a download href='{}'>Télécharger le maître autorisé</a></p>", esc(preview), esc(&a.label), esc(original)));
        } else {
            out.push_str("<p>Fichier exclu : droit ou format à vérifier.</p>");
        }
        out.push_str(&format!(
            "<small>{} · {} · ID {}</small></div>",
            esc(a.role.label_fr()),
            esc(a.variant.label_fr()),
            esc(&a.id)
        ));
    }
    out.push_str("</div><h3>Palette de référence</h3><div class=grid>");
    for c in &d.colors {
        out.push_str("<div>");
        if parse_hex(&c.hex).is_some() {
            out.push_str(&format!(
                "<span class=swatch style='background:{}'></span>",
                esc(&c.hex)
            ));
        }
        out.push_str(&format!(
            "<strong>{}</strong> · {}{}",
            esc(&c.name),
            esc(&c.hex),
            para(&c.notes)
        ));
        if let Some(v) = &c.cmyk {
            out.push_str(&para(&format!(
                "CMYK {} / {} / {} / {} · {}",
                v.c, v.m, v.y, v.k, v.profile
            )));
        }
        if !c.pantone.is_empty() {
            out.push_str(&para(&format!("Pantone : {}", c.pantone)));
        }
        if !c.ral.is_empty() {
            out.push_str(&para(&format!("RAL : {}", c.ral)));
        }
        out.push_str("</div>");
    }
    out.push_str("</div><h3>Typographies</h3><table><tr><th>Famille</th><th>Usage</th><th>Graisses</th><th>Licence</th></tr>");
    for f in &d.fonts {
        let license = if safe_url(&f.license_url) {
            format!(
                "<a rel=noreferrer href='{}'>Acquérir / consulter</a>",
                esc(&f.license_url)
            )
        } else {
            "À renseigner".into()
        };
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{license}</td></tr>",
            esc(&f.family),
            esc(&f.usage),
            esc(&f.weights.join(", "))
        ));
    }
    out.push_str("</table><small>Les fichiers de police sont exclus de cette livraison. Les familles indiquées ne sont pas installées ni chargées par ce guide.</small>");
    for (label, text) in [
        ("Protection", &d.rules.clear_space),
        ("Taille minimale", &d.rules.min_size),
        ("Fonds autorisés", &d.rules.allowed_backgrounds),
    ] {
        out.push_str(&format!("<h3>{label}</h3>{}", para(text)));
    }
    out.push_str("<h3>Interdits</h3><ul>");
    for item in &d.rules.forbidden {
        out.push_str(&format!("<li>{}</li>", esc(&item.text)));
    }
    out.push_str("</ul></section>");
    if !quick {
        let hash = fingerprint(d);
        for spec in applicable(&d.guide) {
            let c = d.guide.chapters.get(spec.id).cloned().unwrap_or_default();
            let state = if matches!(c.state, ReviewState::Validated | ReviewState::NotApplicable)
                && c.approved_fingerprint != hash
            {
                "Validation obsolète"
            } else {
                c.state.label()
            };
            out.push_str(&format!(
                "<section id='{}'><div class=eyebrow>{} · {}</div><h2>{}</h2>",
                spec.id,
                spec.id,
                state,
                esc(spec.title)
            ));
            if c.state == ReviewState::NotApplicable {
                out.push_str(&para(&c.reason));
            } else {
                for key in spec.fields {
                    out.push_str(&format!(
                        "<h3>{}</h3>{}",
                        esc(key),
                        para(
                            c.fields
                                .get(*key)
                                .map(String::as_str)
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or("À documenter")
                        )
                    ));
                }
            }
            for id in &c.asset_ids {
                if let Some((_, image)) = resources.get(id) {
                    out.push_str(&format!("<img style='max-width:100%;max-height:240px' src='{}' alt='Ressource liée {}'>", esc(image), esc(id)));
                }
            }
            if !c.reviewer.is_empty() {
                out.push_str(&format!(
                    "<small>Revue locale : {} · {}</small>",
                    esc(&c.reviewer),
                    esc(&c.reviewed_at)
                ));
            }
            out.push_str("</section>");
        }
    }
    out.push_str("<section><h2>Associations contrôlées</h2><table><tr><th>Contexte</th><th>Premier plan / fond</th><th>Ratio</th><th>Seuil</th></tr>");
    for pair in &d.guide.contrast_pairs {
        let a = d.colors.iter().find(|c| c.id == pair.foreground_id);
        let b = d.colors.iter().find(|c| c.id == pair.background_id);
        let ratio = a.zip(b).and_then(|(a, b)| contrast(&a.hex, &b.hex));
        out.push_str(&format!(
            "<tr><td>{}</td><td>{} / {}</td><td>{}</td><td>{}:1</td></tr>",
            esc(&pair.context),
            esc(a.map(|c| c.name.as_str()).unwrap_or("supprimé")),
            esc(b.map(|c| c.name.as_str()).unwrap_or("supprimé")),
            ratio
                .map(|v| format!("{v:.3}:1"))
                .unwrap_or_else(|| "invalide".into()),
            if pair.large_text_or_graphic {
                "3"
            } else {
                "4.5"
            }
        ));
    }
    out.push_str("</table></section><section><h2>Périmètre et contrôle</h2>");
    out.push_str(&para(&format!(
        "Responsable : {}. Impression : {}. Modules activés : {}.",
        d.guide.owner,
        if d.guide.print_required {
            "oui"
        } else {
            "hors périmètre"
        },
        d.guide.modules.join(", ")
    )));
    out.push_str("<ul>");
    for issue in report.blockers.iter().chain(&report.warnings) {
        out.push_str(&format!(
            "<li>{}: {}</li>",
            esc(&issue.code),
            esc(&issue.message)
        ));
    }
    out.push_str("</ul></section></main><footer>Produit localement par Oracle Brand Kit. Guide HTML autonome et imprimable ; la mise en page PDF dépend du navigateur. Aucun service distant n'est requis.</footer></html>");
    out
}

pub(crate) fn no_links(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(m) => {
                #[cfg(windows)]
                let linked = {
                    use std::os::windows::fs::MetadataExt;
                    m.file_attributes() & 0x400 != 0
                };
                #[cfg(not(windows))]
                let linked = m.file_type().is_symlink();
                if linked {
                    return Err("Livraison refusée : lien ou jonction dans la destination.".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

/// Each export gets a new immutable folder. Nothing is written to master sources or old deliveries.
/// Drafts exclude unauthorized files; final exports stop on any failed check or render.
pub fn export(
    project: &Project,
    engines: &crate::render::Engines,
    draft: bool,
) -> Result<PathBuf, String> {
    export_bundle(project, engines, draft, false)
}

/// A portable client space, generated from the same checked guide and permissions.
/// This is a revision snapshot, not a hosted or synchronized portal.
pub fn export_hub(project: &Project, engines: &crate::render::Engines, draft: bool) -> Result<PathBuf, String> {
    export_bundle(project, engines, draft, true)
}

fn export_bundle(project: &Project, engines: &crate::render::Engines, draft: bool, hub: bool) -> Result<PathBuf, String> {
    let report = check(project);
    if !draft && !report.complete() {
        return Err(format!(
            "Guide non livrable : {} blocage(s). Consulter le contrôle ou exporter un brouillon.",
            report.blockers.len()
        ));
    }
    let root = project.dir.join("Livrables").join(if hub { "Espace-client" } else { "Brand-guides" });
    no_links(&root)?;
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4();
    let stage = root.join(format!(".preparation-{id}"));
    std::fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| {
        std::fs::create_dir(stage.join("assets")).map_err(|e| e.to_string())?;
        let mut resources = BTreeMap::new();
        let mut exclusions = Vec::new();
        for (i, a) in project.data.assets.iter().enumerate() {
            if !authorized(project, &a.id) {
                exclusions.push(a.id.clone());
                continue;
            }
            let source = project.resolve(&a.source);
            let ext = source
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !["svg", "png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                exclusions.push(a.id.clone());
                continue;
            }
            // Snapshot first, then recheck the bytes against the permission. Render this snapshot.
            let original = format!("assets/{:03}-master.{ext}", i + 1);
            let preview = format!("assets/{:03}-preview.png", i + 1);
            std::fs::copy(&source, stage.join(&original)).map_err(|e| e.to_string())?;
            let expected = project
                .data
                .guide
                .rights
                .get(&a.id)
                .map(|r| r.source_hash.as_str())
                .unwrap_or_default();
            if hash_file(&stage.join(&original))? != expected {
                return Err(format!(
                    "{} a changé pendant la préparation ; renouveler l'autorisation.",
                    a.label
                ));
            }
            let rendered = if ext == "svg" {
                crate::render::render_svg(&stage.join(&original), 1.0)
            } else {
                crate::render::rasterize(engines, &stage.join(&original), 1.0)
            };
            match rendered {
                Ok(img) => img.save(stage.join(&preview)).map_err(|e| e.to_string())?,
                Err(e) if draft => {
                    std::fs::remove_file(stage.join(&original)).map_err(|e| e.to_string())?;
                    exclusions.push(format!("{} : {e}", a.id));
                    continue;
                }
                Err(e) => return Err(format!("{} : aperçu impossible : {e}", a.label)),
            }
            resources.insert(a.id.clone(), (original, preview));
        }
        for (file, quick) in [("guide.html", false), ("fiche-reference.html", true)] {
            std::fs::write(
                stage.join(file),
                html(project, &report, draft, &resources, quick),
            )
            .map_err(|e| e.to_string())?;
        }
        std::fs::write(
            stage.join("controle.json"),
            serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mut hashes = BTreeMap::new();
        for (original, preview) in resources.values() {
            for file in [original, preview] {
                hashes.insert(file.clone(), hash_file(&stage.join(file))?);
            }
        }
        for file in ["guide.html", "fiche-reference.html", "controle.json"] {
            hashes.insert(file.into(), hash_file(&stage.join(file))?);
        }
        if hub {
            let mut page = format!("<!doctype html><html lang=fr><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><meta http-equiv=Content-Security-Policy content=\"default-src 'none'; img-src 'self'; style-src 'unsafe-inline'\"><title>{} — Espace client</title><style>body{{background:#1B1A17;color:#fff;font:18px system-ui;max-width:1000px;margin:60px auto;padding:24px}}a{{color:#ff6e86}}section{{border:1px solid #666;padding:24px;margin:24px 0}}img{{background:white;max-width:240px;max-height:150px}}small{{color:#ccc}}</style><h1>{}</h1><p>Version {} · {}</p><p>{}</p><section><h2>Documents</h2><p><a href='guide.html'>Consulter le guide de marque</a></p><p><a href='fiche-reference.html'>Fiche de référence</a></p><p><a download href='manifest.json'>Inventaire et empreintes des fichiers</a></p></section><section><h2>Ressources autorisées</h2>", esc(&project.data.brand_name), esc(&project.data.brand_name), esc(&project.data.version), esc(&crate::now_rfc3339()), if draft { "BROUILLON — à relire, ressources non autorisées exclues." } else { "Guide validé pour le périmètre déclaré." });
            for a in &project.data.assets {
                if let Some((original, preview)) = resources.get(&a.id) {
                    page.push_str(&format!("<article><h3>{}</h3><img src='{}' alt='{}'><p><a download href='{}'>Télécharger le maître</a></p><small>{} · {}</small></article>", esc(&a.label), esc(preview), esc(&a.label), esc(original), esc(a.role.label_fr()), esc(a.variant.label_fr())));
                }
            }
            page.push_str("</section><small>Copie datée hors ligne. Aucun hébergement, compte client, suivi ou mise à jour automatique. Régénérer et transmettre une nouvelle révision après modification. Les polices ne sont pas redistribuées.</small></html>");
            std::fs::write(stage.join("index.html"), page).map_err(|e| e.to_string())?;
            hashes.insert("index.html".into(), hash_file(&stage.join("index.html"))?);
        }
        let manifest = serde_json::json!({ "schemaVersion": 1, "projectId": project.data.project_id, "brand": project.data.brand_name,
            "version": project.data.version, "createdAt": crate::now_rfc3339(), "draft": draft, "completeForScope": !draft && report.complete(),
            "fingerprint": fingerprint(&project.data), "resourcesById": resources, "excluded": exclusions,
            "fontFilesIncluded": false, "offlineClientSpace": hub, "sha256": hashes });
        std::fs::write(
            stage.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let folder = root.join(format!(
            "{}-{id}",
            if draft { "brouillon" } else { "guide-valide" }
        ));
        std::fs::rename(&stage, &folder).map_err(|e| e.to_string())?;
        Ok(folder)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&stage);
    }
    result
}
