# Oracle Brand Kit et Mockup Studio

**Brand Kit** : la source de vérité d'une identité de marque — logos rangés par rôle, palette,
typographies, règles d'usage — et la fabrication des fichiers à livrer au client.
**Mockup Studio** : les logos de la marque posés sur des photos de supports (quatre coins, masques,
premier plan, surcouches), strictement 2D. Local, sans compte, sans IA : chaque logo, variante,
couleur, photo et masque vient du designer ; exports et rendus ne tournent que sur demande.

## Contenu

| Dossier | Rôle |
|---|---|
| `core/` (crate `oracle-brand`) | modèle `brand.json` (schéma v1, migration), projet, rendu, export, manifest/README ; modèles et scènes de mockup (`mockup`), composition (`composite`) ; CLI `oracle-brand` |
| `style/` (`oracle-style`) | la DA commune (Lexend, clair/sombre, arrondi 4 px, rouge Oracle) |
| `kit/` (`oracle-brand-kit`) | l'application Brand Kit |
| `mockup/` (`oracle-mockup-studio`) | l'application Mockup Studio |

## Commandes

```sh
cargo test --workspace                        # 25 tests : modèle, projet, export, mockups, deux interfaces
cargo build --release --workspace             # target/release/oracle-brand-kit.exe, oracle-mockup-studio.exe, oracle-brand.exe
python ../tools/demo_brand_kit.py             # démo Brand Kit de bout en bout (données fictives)
python ../tools/demo_mockup.py                # démo Mockup Studio de bout en bout (photo de synthèse)
UPDATE_SNAPSHOTS=1 cargo test -p oracle-brand-kit -p oracle-mockup-studio   # régénère les captures tests/snapshots/
```

Le rendu passe par `oracle-vector-cli` (variable `ORACLE_VECTOR_CLI`, app installée, ou build de
développement voisin) et `oracle-edit-cli` pour les PSD. Sans Oracle Vector, seuls les SVG restent
exportables (rendu resvg).

## Mockup Studio

- Un **modèle** est un dossier : `template.json`, une copie de la photo (l'original n'est jamais
  écrit), et les masques et surcouches préparés par le designer. Zones : quatre coins sur la photo,
  proportions réelles (85 × 55), masque, fusion (normal, produit, superposition claire), opacité.
  Un masque de premier plan remet devant les designs ce qui doit l'être (une main, un objet).
- Une **scène** appartient au projet de marque (`Mockups/Scènes/*.json`) : chaque zone reçoit un
  logo du Brand Kit (lié par identifiant, avec son rôle et sa variante pour resservir à une autre
  marque) ou une image, sur une couleur de la palette ou une valeur. Placement : contenir, remplir
  (recadrer) ou étirer, taille, rotation, décalage. Rendu en PNG, JPEG ou WebP dans `Mockups/`.
- Chaque rendu note l'empreinte de tout ce qui l'a produit : un logo, un masque ou un réglage
  modifié rend la scène « à mettre à jour » (avec la raison), dans Mockup Studio comme sur la fiche
  du logo dans Brand Kit ; rien n'est re-rendu sans clic.
- La perspective est l'homographie d'Oracle Edit (`oracle-edit-algo`), rien n'est deviné à partir
  de la photo (ni plis, ni éclairage). Photos PSD : lues aplaties par Oracle Edit.

## Garanties

- Les fichiers maîtres ne sont jamais écrits : ils sont lus, et leur empreinte SHA-256 est notée.
- Chaque fichier livré est écrit de façon atomique (fichier temporaire puis renommage).
- Une livraison marquée « livrée » n'est plus jamais réécrite : le prochain export crée `v1-2`, `v1-3`…
- Un JPEG exige un fond ; un maître non vectoriel n'est jamais exporté en SVG/PDF.
- CMJN, Pantone et RAL sont des champs saisis, jamais calculés ; une valeur CMJN nomme son profil ICC.
- Les fichiers de police ne sont pas livrés (le README du livrable le dit).

## Licences

Code : MIT OR Apache-2.0. Polices de l'interface : Lexend et Inter, SIL OFL 1.1
(`style/assets/OFL-*.txt`). Dépendances : egui/eframe, image, resvg, sha2, uuid, serde, rfd, dirs,
winresource — MIT et/ou Apache-2.0 (resvg : Apache-2.0 OR MIT) ; `oracle-edit-algo` (Oracle Edit,
MIT OR Apache-2.0, avec rustfft et rayon : MIT OR Apache-2.0) ; `oracle-library-ui` (Oracle).
Données de test : photo de carte et logos dessinés par le code, sans asset tiers.
