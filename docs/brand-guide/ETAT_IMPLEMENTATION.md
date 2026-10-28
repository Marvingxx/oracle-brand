# Oracle Brand Kit : état réalisé et limites

**27 octobre 2026 — version de développement**, dans `C:\Users\33652\Oracle-suite\oracle-brand`. Le dossier historique `C:\Users\33652\Desktop\Oracle` est absent. Les modifications déjà présentes ont été conservées ; cette livraison ajoute le guide au projet existant.

## Fonctions réalisées

- Onglet **Brand guide** : périmètre, responsable, personne qui valide et destination impression ; douze chapitres communs et huit modules activables.
- Décisions par chapitre, ressources liées, brouillon, validation nominative datée et non-applicabilité justifiée pour les modules conditionnels.
- Réutilisation des logos, couleurs, polices et règles de `brand.json`, sans second catalogue. Une modification du contenu partagé rend les validations précédentes obsolètes ; l’invalidation est volontairement globale.
- Autorisations de remise par ressource avec indication de preuve, nom, date et empreinte du fichier. Remplacer le fichier invalide l’autorisation correspondante.
- Associations de couleurs liées à la palette, contexte déclaré et calcul de contraste sRGB opaque. Seuil de 4,5 pour texte courant, 3 pour grand texte/éléments graphiques, sans arrondir avant décision. Ce contrôle ne constitue pas une certification d’accessibilité.
- Contrôle avant remise : contenus, validations, références, présence et droits des fichiers, formats, palette, informations typographiques, règles et, si impression déclarée, CMJN fourni avec profil.
- Export brouillon possible avec blocages visibles ; export final bloqué tant que le contrôle échoue. Les fichiers non autorisés sont exclus des brouillons.
- Dossier unique dans `Livrables/Brand-guides/` : `guide.html`, `fiche-reference.html`, `controle.json`, `manifest.json`, maîtres autorisés et aperçus PNG. Les originaux et anciennes sorties ne sont pas écrasés. Le manifeste inventorie les empreintes et exclusions.
- Présentation HTML avec fond `#1B1A17`, accent `#D31C3B`, palette de la marque documentée séparément et styles d’impression. Le thème partagé de l’application reste celui déjà présent dans la suite.

Les chapitres couvrent périmètre, plateforme, logos, couleurs, typographies, univers visuel, composition, voix, accessibilité, applications, production et maintenance. Les modules sont architecture/co-branding, numérique, motion, audio, packaging/signalétique, données, internationalisation et mascotte. Leurs champs structurent les décisions ; ils ne créent pas automatiquement des animations, maquettes, composants ou illustrations.

## Démarrer et essayer

Depuis la racine du dépôt, avec Cargo installé :

```powershell
cargo build -p oracle-brand -p oracle-brand-kit --bins
& .\target\debug\oracle-brand-kit.exe --guide .\outputs\brand-guide-demo-2026-10-27
```

La démonstration est explicitement fictive : un symbole SVG original, une palette d’écran, une référence à Inter et douze chapitres pédagogiques non validés. Aucun visa client n’est fabriqué. Le brouillon déjà généré est accessible sous :

`outputs/brand-guide-demo-2026-10-27/Livrables/Brand-guides/brouillon-9fed55f9-1b2a-48a5-8651-f50d4d10c215/guide.html`

Pour un autre projet fictif, l’exemple refuse d’écraser un projet existant :

```powershell
cargo run -p oracle-brand --example brand_guide_demo -- CHEMIN_NOUVEAU_DOSSIER
```

Commandes complémentaires :

```powershell
.\target\debug\oracle-brand.exe guide-check CHEMIN_PROJET
.\target\debug\oracle-brand.exe guide-export CHEMIN_PROJET --draft
.\target\debug\oracle-brand.exe guide-export CHEMIN_PROJET
```

Le contrôle retourne un échec si le projet est incomplet. L’export sans `--draft` exige l’absence de blocages. Une sortie réussie est référencée dans les documents du projet depuis l’interface et la commande d’export.

## Vérification

Les preuves et le résultat final des commandes sont consignés dans [le compte rendu de vérification](../../outputs/brand-guide-verification/VERIFICATION.md). Les tests vérifient notamment migrations, blocages de livraison, empreintes, ressources exclues, seuils de contraste, conservation des originaux et absence de redistribution des polices. Le test de l’interface parcourt aussi les anciennes pages et produit réellement leurs seize fichiers d’export.

L’image `outputs/brand-guide-verification/brand-guide-editor.png` provient du rendu du banc de test de l’interface. C’est une vérification du rendu et des interactions du banc, pas une capture de l’application sur le Bureau. L’ouverture d’un processus natif est une preuve de lancement, pas une recette manuelle complète.

## Limites et compatibilité

Le schéma du projet passe en **v2**. Les versions v0/v1 sont migrées en mémoire à l’ouverture ; seul un enregistrement explicite persiste la migration. Les champs inconnus à la racine sont conservés. La conservation de tous les champs inconnus imbriqués n’est pas garantie. Aucun projet utilisateur existant n’a été migré sur disque pour cette démonstration. Une ancienne application peut refuser un projet sauvegardé en v2 : les autres applications installées n’ont pas été mises à jour ni testées avec ce schéma.

Le nouvel export du guide accepte les maîtres **SVG, PNG, JPEG et WebP**. Les capacités plus larges de l’ancien moteur de kit n’étendent pas automatiquement cette liste. Tous les assets du projet doivent actuellement être autorisés et pris en charge pour un export final ; il n’existe pas encore de sélection d’exclusion définitive par livraison. Les ressources externes éventuellement référencées dans un SVG ne font pas l’objet d’un contrôle exhaustif : vérifier leur autonomie avant livraison hors ligne.

Les champs remplis et un nom de validation ne prouvent ni la qualité stratégique ni la validité juridique d’un document. Les liens de licence des polices sont des références à vérifier ; le guide n’embarque aucun fichier de police. Les valeurs CMJN et profils sont déclaratifs : aucune conversion ICC ni épreuve d’impression n’est produite.

Le guide HTML est livré. Le nouvel onglet **Studio** ajoute les grilles de présentation réglables, les présentations de pistes, deux modèles Bento et un espace client hors ligne ; voir [la conception du Studio](CONCEPTION_STUDIO.md) et [sa vérification](../../outputs/brand-studio-verification/VERIFICATION.md). Un générateur PDF natif, un document AI/INDD éditable, la déduction automatique des courbes de construction d’un logo et un portail client cloud restent à développer. L’impression depuis un navigateur n’a pas fait l’objet d’une recette. Les contrôles relisent les fichiers ; les performances sur une très grande bibliothèque restent à mesurer.

Cette livraison ne modifie ni les intégrations Adobe, ni les réglages et curseurs Windows/Photoshop/Affinity. Aucun document de création utilisateur n’a été enregistré ou fermé.
