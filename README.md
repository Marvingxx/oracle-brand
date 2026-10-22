# Oracle Brand Kit

La source de vérité d'une identité de marque — logos rangés par rôle, palette, typographies,
règles d'usage — et la fabrication des fichiers à livrer au client. Local, sans compte, sans IA :
chaque logo, variante, couleur et règle vient du designer ; les exports ne tournent que sur demande.

## Contenu

| Dossier | Rôle |
|---|---|
| `core/` (crate `oracle-brand`) | modèle `brand.json` (schéma v1, migration), projet, rendu, export, manifest/README ; CLI `oracle-brand` |
| `kit/` (`oracle-brand-kit`) | l'application (egui, DA Oracle : Lexend, clair/sombre, arrondi 4 px) |

## Commandes

```sh
cargo test --workspace                        # 15 tests : modèle, projet, export réel, interface
cargo build --release --workspace             # target/release/oracle-brand-kit.exe et oracle-brand.exe
python ../tools/demo_brand_kit.py             # démo de bout en bout (données fictives)
UPDATE_SNAPSHOTS=1 cargo test -p oracle-brand-kit   # régénère les captures kit/tests/snapshots/
```

Le rendu passe par `oracle-vector-cli` (variable `ORACLE_VECTOR_CLI`, app installée, ou build de
développement voisin) et `oracle-edit-cli` pour les PSD. Sans Oracle Vector, seuls les SVG restent
exportables (rendu resvg).

## Garanties

- Les fichiers maîtres ne sont jamais écrits : ils sont lus, et leur empreinte SHA-256 est notée.
- Chaque fichier livré est écrit de façon atomique (fichier temporaire puis renommage).
- Une livraison marquée « livrée » n'est plus jamais réécrite : le prochain export crée `v1-2`, `v1-3`…
- Un JPEG exige un fond ; un maître non vectoriel n'est jamais exporté en SVG/PDF.
- CMJN, Pantone et RAL sont des champs saisis, jamais calculés ; une valeur CMJN nomme son profil ICC.
- Les fichiers de police ne sont pas livrés (le README du livrable le dit).

## Licences

Code : MIT OR Apache-2.0. Polices de l'interface : Lexend et Inter, SIL OFL 1.1
(`kit/assets/OFL-*.txt`). Dépendances : egui/eframe, image, resvg, sha2, uuid, serde, rfd, dirs,
winresource — MIT et/ou Apache-2.0 (resvg : Apache-2.0 OR MIT).
