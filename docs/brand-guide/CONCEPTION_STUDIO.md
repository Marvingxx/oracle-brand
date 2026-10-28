# Oracle Studio — conception et première version fonctionnelle

27 octobre 2026. Implémentation indépendante dans Oracle Brand Kit. Les pages publiques d’Akrivi ont servi à identifier des besoins, sans consulter son code, ses binaires ou ses contenus privés. Aucun plugin Illustrator Akrivi n’est reproduit ou intégré.

[Ouvrir les démonstrations fictives](../../outputs/brand-studio-verification/index.html) · [Vérification et limites constatées](../../outputs/brand-studio-verification/VERIFICATION.md).

## Un projet commun plutôt que six bibliothèques

L’onglet **Studio** rassemble les outils autour du `brand.json` existant : logos, palette, références typographiques, règles, droits et version. Les recettes sont sauvegardées dans un champ `studio` facultatif du schéma v2. Ouvrir une page ne modifie pas le projet ; une édition utilise l’enregistrement automatique existant. Les champs inconnus de la recette et des pistes sont conservés.

Le Brand guide reste l’espace documentaire. Les compositions n’invalident pas les validations éditoriales du guide ; les changements d’identité partagée continuent de les invalider. Les exports restent des révisions indépendantes, jamais une synchronisation automatique des dossiers précédemment remis.

| Outil | Parcours disponible | Sortie et limites |
|---|---|---|
| Grilles de logo | Choisir un logo ; régler l’unité et la protection ; prévisualiser | Planche de présentation, axes et zone autour du cadre. Ne déduit pas les courbes ni la construction d’origine. |
| Présentations | Ajouter jusqu’à 50 pistes ; choisir leurs logos ; nommer, argumenter, ordonner, supprimer et noter une décision interne | Une planche par piste et une page HTML avec l’argumentaire intégral. Toutes les pistes et leurs commentaires sont remis, même celles écartées ; relire avant transmission. Aucun visa client n’est créé. |
| Bento | Choisir le logo et le modèle Éditorial ou Mosaïque ; saisir une phrase | Reprend au plus 8 couleurs valides et 3 références typographiques. Composition fixe, sans éditeur libre ni génération de mockups. |
| Exports de logos | Accéder aux presets, à l’export et aux livraisons existants | Réutilise le moteur actuel ; disponibilité des formats selon les convertisseurs détectés. |
| Brand guide | Renseigner périmètre, chapitres, preuves et validations | Guide HTML, fiche de référence et ressources autorisées ; détails dans le bilan général. |
| Espace client | Exporter un brouillon ou une révision finale contrôlée | Dossier web hors ligne avec accueil, guide, fiche, maîtres autorisés et manifeste. Pas de compte, hébergement, lien public vivant, statistiques ni synchronisation distante. |

## Présentation et fichiers

Les planches utilisent le charbon `#1B1A17`, le rouge `#D31C3B` et le blanc comme cadre Oracle. La palette du client reste distincte et apparaît dans la planche Bento. Le thème partagé de l’application est conservé.

Chaque export Studio crée un dossier unique sous `Livrables/Studio/` : `index.html`, `recette.json`, `manifest.json`, une ou plusieurs planches SVG et leurs PNG **1200 × 800**. Le SVG permet de modifier la composition, mais ses logos sont des aperçus PNG incorporés : ce n’est pas un logo vectoriel reconstruit. Les originaux restent dans le projet. Les textes des planches utilisent Arial ou un repli local ; les noms des polices du projet sont des références, pas des spécimens. Aucun fichier de police n’est distribué. Les textes longs peuvent être abrégés dans la planche ; le titre, l’argumentaire et la phrase sont conservés intégralement dans le HTML ou la recette.

Le Studio accepte pour ses aperçus SVG, PNG, JPEG, WebP, TIFF, BMP et GIF selon le décodeur ; GIF : image fixe. Les essais décrits dans la vérification portent sur les formats effectivement testés, pas sur une compatibilité universelle. PDF, EPS, AI, PSD et fichiers Affinity nécessitent une copie convertie pour le Studio. L’ancien moteur d’export conserve ses propres capacités. Les SVG à ressources externes ou à polices particulières demandent une vérification visuelle.

## Remise et intégrité

La prévisualisation locale reste possible avant validation des droits. La remise des planches exige pour chaque source utilisée une autorisation avec preuve, personne, date et empreinte actuelle. Le moteur prend une copie temporaire contrôlée, produit les planches puis retire cette copie avant publication du dossier. Un échec supprime la préparation incomplète. Les anciennes sorties et les maîtres sont préservés ; le manifeste référence les empreintes des sources et des sorties. Les recettes Grille et Bento n’embarquent pas les commentaires des autres pistes.

L’espace client utilise les mêmes contrôles que le guide. Un brouillon indique les blocages et exclut les ressources non autorisées ; la remise finale exige un guide complet et des validations à jour. La page d’accueil n’ajoute pas de service en ligne.

## Utilisation

Depuis la racine du dépôt :

```powershell
cargo build -p oracle-brand -p oracle-brand-kit --bins
.\target\debug\oracle-brand-kit.exe --studio CHEMIN_PROJET
.\target\debug\oracle-brand.exe studio-export CHEMIN_PROJET grid
.\target\debug\oracle-brand.exe studio-export CHEMIN_PROJET bento
.\target\debug\oracle-brand.exe studio-export CHEMIN_PROJET presentations
.\target\debug\oracle-brand.exe hub-export CHEMIN_PROJET --draft
```

Dans l’application, ajouter les logos dans **Logos**, sélectionner une ressource dans **Studio** et enregistrer ses autorisations dans **Brand guide** avant d’exporter les planches. Le bouton du dernier export ouvre sa page HTML. Les révisions antérieures restent consultables dans le dossier `Livrables` ; le bouton garde uniquement le dernier résultat de la session.

Pour créer une nouvelle démonstration fictive, sans écraser un projet :

```powershell
cargo run -p oracle-brand --example brand_studio_demo -- CHEMIN_NOUVEAU_DOSSIER
```

## Suites possibles

La version présente permet déjà de travailler et de produire des fichiers. Un éditeur de mise en page libre, les grilles fondées sur les tracés vectoriels, des spécimens typographiques fidèles, des supports de présentation supplémentaires, le PDF natif et un portail hébergé restent des extensions distinctes. Aucun de ces éléments n’est annoncé comme livré.

Références fonctionnelles : [Akrivi Studio](https://www.akrivi.studio/), [Logo Exports](https://www.akrivi.studio/logo-exports), [Brand Hub](https://www.akrivi.studio/brand-hub). Les fonctions commerciales d’Akrivi ne prouvent pas une parité de cette première version Oracle.
