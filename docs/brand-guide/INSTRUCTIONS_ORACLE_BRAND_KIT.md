# Instructions produit et développement — Oracle Brand Kit complet

**Cahier de spécifications — 27 octobre 2026**  
Statut : **cahier cible, partiellement implémenté le 27 octobre 2026**, fondé sur l’examen initial du code et sur la [recherche de référence](RECHERCHE_BRAND_GUIDE.md). Voir l’[état vérifié](ETAT_IMPLEMENTATION.md) pour les fonctions disponibles et l’[étude d’Akrivi](ETUDE_AKRIVI.md) pour la comparaison fonctionnelle. Les tableaux d’existant ci-dessous décrivent la situation avant cette évolution, notamment le schéma v1 ; le nouveau schéma est v2.

## 1. Consigne générale à transmettre au développement

Faire évoluer Oracle Brand Kit pour qu’une même identité structurée permette de préparer, vérifier et livrer un **brand kit utilisable** et un **brand guide complet pour un périmètre explicite**. Réutiliser l’application, le modèle de projet, les outils de rendu, la Bibliothèque Oracle et les mécanismes de livraison existants.

Le logiciel doit aider le designer à décider, documenter et assembler. Il ne doit pas inventer une stratégie, des variantes de logo, des références d’impression, des licences ou une validation. « Générer un guide » signifie mettre en forme des contenus et ressources fournis et validés, conformément au fonctionnement local et sans IA du projet.

Commencer par lire le [README](../../README.md), le [modèle](../../core/src/model.rs), la [gestion des projets](../../core/src/project.rs), les [exports](../../core/src/export.rs), les [pages Brand Kit](../../kit/src/pages.rs) et la [CLI](../../core/src/bin/oracle-brand.rs). Préserver les modifications déjà présentes dans le dépôt. Implémenter par étapes avec preuves de fonctionnement ; ne pas annoncer une fonction à partir de la seule présence d’un champ ou d’une maquette.

## 2. Existant observé et écarts à combler

Les constats suivants proviennent de la lecture des sources, pas d’une recette exécutée de l’application.

| Sujet | Existant observé | Évolution demandée |
|---|---|---|
| Projet | `brand.json`, schéma v1, identité, client, version, statut et notes | Conserver ces données et ajouter le périmètre, les validations et les contenus du guide. |
| Logos | Assets avec ID, rôle, variante, chemin et empreinte du fichier source | Relier chaque règle et exemple au bon asset ; valider les besoins de variantes. |
| Couleurs | HEX, CMYK avec profil, références Pantone/RAL et notes | Ajouter rôles, associations, contextes et résultats de contrôle. |
| Polices | Famille, graisses, usage, fichiers locaux et licence | Structurer hiérarchies, remplacement, caractères et conditions d’usage. |
| Règles | Zone de protection, taille minimale, fonds et interdits | Conserver le texte existant et compléter par des mesures, exemples et décisions vérifiables. |
| Guide | `guide_documents` contient des références de documents | Ajouter un contenu structuré et un véritable assemblage de guide ; ces références seules ne constituent pas un générateur. |
| Exports | Préréglages, plan, exports, manifeste, README et livraisons | Étendre le plan et la livraison aux guides, fiches et modèles réellement produits. |
| Mockups | Scènes liées aux ressources, rendus et détection d’obsolescence | Utiliser les scènes comme exemples ; distinguer illustration et modèle modifiable. |
| Bibliothèque | Intégration annoncée à Oracle Assets et Oracle Font | Réutiliser les identifiants et références ; aucune seconde bibliothèque parallèle. |

La page de règles annonce une utilisation dans le README et le guide. L’examen réalisé ne suffit pas à établir l’existence d’un export de guide complet. Vérifier les capacités au moment de l’implémentation plutôt que reprendre cette annonce comme preuve.

## 3. Principes à préserver

- Fonctionnement local, sans compte et sans IA ; contenus et créations fournis par le designer.
- Originaux préservés, contrôlés par empreinte ; aucun remplacement silencieux d’un maître.
- Rendus et exports déclenchés sur demande explicite. Marquer une sortie obsolète sans la recalculer automatiquement.
- Écritures sûres et livraisons déjà remises immuables ; nouvelle version pour une nouvelle remise.
- Une Bibliothèque Oracle centrale, avec les mêmes collections et ressources dans les autres outils.
- Valeurs CMYK, Pantone et RAL renseignées ou validées par le designer ; pas d’équivalence inventée à partir d’un HEX.
- Respect des capacités de rendu réelles : ne pas transformer une image en prétendu logo vectoriel.
- Préservation des usages existants de Photoshop et des autres intégrations. Ces instructions ne demandent aucun changement de curseur ou d’interface d’une application tierce.

La politique documentée actuelle exclut la livraison des fichiers de police. Le commentaire du champ `redistributable` évoque une autorisation conditionnelle : cette divergence doit être clarifiée sans activer la redistribution. Un booléen ne suffit pas à prouver une licence.

## 4. Définir ce que « complet » veut dire

À la création ou à la reprise d’un projet, recueillir : destinataires, supports, pays, langues, canaux, livrables, usages d’impression, logiciels de destination et responsable de validation. Proposer des profils de départ, mais laisser le designer ajuster les exigences.

Chaque rubrique possède un état éditorial : **manquante**, **brouillon**, **validée** ou **non applicable**. « Non applicable » doit conserver une justification et son auteur. Une rubrique vide ne devient jamais non applicable automatiquement.

Séparer cet état des problèmes techniques ou juridiques : fichier absent, ressource modifiée, sortie obsolète, droit non établi, contrôle à effectuer, erreur de rendu. Séparer aussi la validation du contenu de la remise effective au client.

Un guide peut être marqué complet lorsque toutes les exigences applicables sont traitées, les contrôles requis réussis et les décisions validées. Afficher les blocages et leurs conséquences. Un pourcentage de remplissage peut aider à naviguer ; il ne peut pas neutraliser une absence de logo requis ou une licence non établie.

Une validation locale signifie « approuvé par la personne indiquée dans ce projet ». Ne pas la présenter comme une signature certifiée ou une autorité juridique.

## 5. Contenus à prendre en compte

Pour chaque ligne, permettre un contenu explicatif, des ressources liées, des exemples, une validation et les réserves éventuelles. Les chapitres de la recherche détaillent le fond attendu.

| ID | Module | Données et résultat attendus | Critère essentiel |
|---|---|---|---|
| BG-01 | Périmètre et gouvernance | Publics du guide, supports, langues, propriétaire, contact, version et exclusions | Le destinataire comprend ce qui est couvert et où se trouve la référence. |
| BG-02 | Plateforme de marque | Mission, public, positionnement, promesse, preuves, personnalité, messages | Les affirmations sont fournies et validées ; les valeurs guident des choix concrets. |
| BG-03 | Logos | Rôles, variantes, usage, protection, tailles, fonds, partenariats, interdits | Chaque règle renvoie au fichier concerné et à une mesure applicable. |
| BG-04 | Couleurs | Rôles, codes, espace, associations, impression et limites | Les combinaisons sont évaluées par usage ; les références d’impression sont explicites. |
| BG-05 | Typographie | Familles, styles, hiérarchie, espacements, caractères, remplacement et licences | Le destinataire sait utiliser ou acquérir les bonnes polices. |
| BG-06 | Images et éléments visuels | Photo, illustration, icônes, motifs, textures et droits | Les règles permettent de choisir de nouvelles ressources cohérentes. |
| BG-07 | Composition | Grilles, marges, rythme, hiérarchie, densité et variantes de format | Les règles résistent aux contenus et formats réellement prévus. |
| BG-08 | Voix et ton | Règles éditoriales, lexique, contexte, exemples et contre-exemples | Une autre personne sait rédiger dans la voix de la marque. |
| BG-09 | Accessibilité | Contraste, couleur, caractères, alternatives et exigences par support | Chaque contrôle est limité à ce qu’il démontre réellement. |
| BG-10 | Applications | Supports prioritaires, exemples, modèles et dépendances | Un modèle promis est présent et modifiable dans la destination annoncée. |
| BG-11 | Production et livraison | Fichiers, paramètres, notice, inventaire, droits et contrôles | Les ressources promises existent et sont utilisables. |
| BG-12 | Maintenance | Responsables, historique, exceptions, remplacements et versions | Une modification peut être propagée sans altérer une livraison passée. |

Modules conditionnels : architecture de marque, partenariat, design system, mouvement, son, packaging, signalétique, données, internationalisation et personnage. Les rendre obligatoires lorsque le périmètre en dépend. Ne pas exiger une section sonore à une marque qui n’en a aucun usage.

## 6. Modèle de données proposé

Cette section est une **proposition d’évolution**, pas le schéma actuellement accepté par l’application. Ne pas fournir un faux `brand.json` présenté comme directement importable.

### Réutiliser les données existantes

Conserver les identifiants d’assets, couleurs, polices, préréglages, scènes et livraisons. Les contenus du guide doivent les référencer plutôt que recopier leurs valeurs dans plusieurs structures indépendantes. Un changement de couleur doit pouvoir rendre les pages et sorties qui en dépendent obsolètes.

Préserver les références existantes de `guide_documents` comme pièces jointes ou guides historiques, sans les assimiler à des chapitres structurés. Préserver les règles libres existantes comme texte importé, à revoir avant validation.

### Ajouter les groupes suivants

| Groupe proposé | Informations minimales |
|---|---|
| `scope` | Supports, audiences, langues, territoires, formats, exclusions, modules requis. |
| `strategy` | Mission, positionnement, promesse, preuves, personnalité et messages. |
| `guide_sections` | ID stable, module, texte, ressources, exemples, état, révision et validation. |
| `logo_rules` | Asset concerné, unité de protection, mesure, minimum écran/impression, fonds et associations. |
| `color_usage` | Références de couleurs, rôles, paires, contexte, transparence et validations. |
| `typography_rules` | Police référencée, rôle, style, taille par contexte, interligne, approche, remplacement et langues. |
| `visual_language` | Photo, illustration, icônes, motifs et règles de composition. |
| `verbal_identity` | Voix, tons par situation, lexique, conventions et exemples validés. |
| `applications` | Support, modèle source, dépendances, sorties attendues, logiciel et essai. |
| `rights_records` | Ressource, provenance, licence ou preuve, droits connus, restrictions, crédits et statut de vérification. |
| `validation_records` | Règle contrôlée, résultat, méthode automatique/manuelle, auteur, date et empreintes concernées. |
| `guide_outputs` | Configuration d’édition, langue, chapitres, couverture, profil de livraison et dépendances. |

Une valeur de taille ou d’espacement doit conserver son unité et son contexte. Une zone de protection peut dépendre d’un repère graphique : stocker cette référence et son illustration. Le texte libre reste disponible pour les cas non structurables.

### Migration et conservation

Une extension incompatible exige une évolution explicite du schéma, par exemple v2 lors de son implémentation. Tester l’ouverture d’un projet v1, sa sauvegarde de secours, la stabilité des identifiants et les références. N’initialiser aucune ancienne donnée comme « validée » sans décision.

Refuser une version de schéma plus récente non comprise. Ne pas supposer que la désérialisation actuelle conserve tous les champs inconnus : vérifier ce comportement et adopter une stratégie qui évite leur perte silencieuse. Aucun changement de schéma n’est exécuté par cette livraison documentaire.

## 7. Parcours utilisateur demandé

Garder une interface simple, intégrée à Brand Kit : **Marque**, **Ressources**, **Règles et guide**, **Applications**, **Vérification**, **Livraison**. Cette organisation est proposée ; l’intégrer aux pages existantes plutôt que créer une application parallèle.

1. Définir le périmètre ou reprendre le projet existant.
2. Choisir les ressources dans la Bibliothèque ou référencer les fichiers déjà présents.
3. Rédiger les règles avec des exemples liés et des champs adaptés au support.
4. Prévisualiser le guide, les applications et les éléments manquants.
5. Corriger les problèmes depuis une liste qui ouvre directement la rubrique concernée.
6. Valider les décisions puis examiner le plan de livraison avant de produire les fichiers.

L’utilisateur peut sauvegarder un projet incomplet. Ne pas bloquer la rédaction parce qu’un contrôle de livraison échoue. Expliquer un problème par sa conséquence et la correction attendue : « police requise absente : le modèle ne pourra pas conserver sa mise en page », par exemple.

Séparer la DA de l’outil de celle des marques : conserver le style Oracle existant ; prévisualiser chaque client avec ses propres couleurs, sans substituer automatiquement le rouge Oracle.

## 8. Contrôles spécialisés

### Logos et fichiers

Vérifier l’existence, l’empreinte, le rôle et la variante. Demander une variante manquante plutôt que recolorer ou simplifier automatiquement le logo. Tester la taille minimale sur la ressource réelle ; ne pas appliquer un minimum universel.

Associer chaque interdiction illustrée à un exemple fourni ou construit explicitement pour la démonstration, clairement identifié. Ne jamais modifier le maître pour fabriquer un exemple.

### Couleurs et contraste

Valider le format des codes et calculer les contrastes sRGB pour les couleurs opaques. La linéarisation de chaque composante sRGB utilise le seuil 0,04045, puis la luminance relative pondérée 0,2126 / 0,7152 / 0,0722 ; le ratio est `(Lmax + 0,05) / (Lmin + 0,05)`. Comparer le ratio non arrondi au seuil requis par le contexte. Références et exceptions : chapitre 10 de la recherche.

Conserver ces cas de contrôle, pour un fond uni et sans transparence :

| Paire | Résultat attendu approximatif | Décision pour texte courant AA |
|---|---:|---|
| `#D31C3B` et `#1B1A17` | 3,313934 | Échec à 4,5:1. |
| `#FFFFFF` et `#D31C3B` | 5,251271 | Réussite à 4,5:1. |
| `#FFFFFF` et `#1B1A17` | 17,402366 | Réussite à 4,5:1. |

La réussite pour du grand texte doit préciser la taille et la graisse. Les fonds variables et images doivent conduire à une vérification du rendu, pas à un résultat automatique trompeur. Les composants et graphiques nécessaires disposent de critères et d’exceptions propres.

Pour l’impression, afficher CMYK et profil saisis ensemble. Sans spécifications suffisantes, ne pas déclarer la livraison prête à imprimer. Cela n’interdit pas de valider une livraison uniquement numérique. Conserver les références de tons directs comme données déclarées, avec leur statut de validation.

### Polices et licences

Contrôler la disponibilité des familles/styles, les caractères nécessaires et les dépendances des modèles. Ne pas annoncer un style absent comme disponible. Faire valider la police de remplacement et vérifier l’effet sur les retours à la ligne.

Les fichiers de police ne sont pas inclus dans les livraisons selon la politique actuelle. Fournir références, liens et instructions d’acquisition. L’incorporation dans un PDF et l’usage web nécessitent des vérifications spécifiques ; ne pas déduire ces droits d’un simple nom de famille ou d’une licence de poste de travail.

L’HTML hors ligne ne doit pas redistribuer clandestinement des fontes via un fichier CSS encodé. Utiliser des ressources autorisées dans le cadre de la politique du produit ou une solution de remplacement explicitement validée, et signaler les variations possibles selon le poste.

### Contenu et cohérence

Détecter les références cassées, rubriques requises vides, exemples non validés, contradictions de version et dépendances modifiées. Ne pas remplacer une information absente par un slogan ou une règle générique. Les noms de champs internes ne doivent pas apparaître comme contenu client.

## 9. Relation avec la Bibliothèque et les logiciels de création

Réutiliser les composants de bibliothèque existants et leurs identifiants. Les collections de travail doivent désigner les mêmes ressources dans Oracle Assets, Brand Kit et les autres intégrations qui utilisent cette bibliothèque. Une livraison peut copier les ressources nécessaires dans un dossier portable ; cette copie est un instantané volontaire.

Ne pas dupliquer l’ensemble de la bibliothèque, créer un index concurrent ou présenter un export statique comme une synchronisation. Une éventuelle collection `.afassets` importée reste un instantané, sauf mécanisme de synchronisation distinct réellement implémenté et vérifié.

Pour les modèles et formats d’échange, afficher une matrice : format, application/version testée, ouverture, édition des textes, calques/objets, polices, effets et limites. Préserver les sources. Un PSD accepté par un logiciel ne prouve ni son rendu exact ni son éditabilité complète.

Les essais dans Photoshop ou Affinity doivent se faire sur des copies et documents de test, sans modifier, fermer ou enregistrer les documents utilisateur déjà ouverts. Cette recette future ne doit pas être présentée comme déjà effectuée.

## 10. Génération du guide et des livrables

### Une seule source de vérité

Composer le guide à partir des données validées et des ressources liées. Le designer reste libre de rédiger et réordonner les explications. Un PDF externe joint peut compléter le projet ; il ne doit pas devenir un second emplacement silencieusement désynchronisé pour les mêmes règles.

Permettre un guide complet, une fiche de prise en main et un kit correspondant au périmètre. Fournir couverture, sommaire, chapitres, références des fichiers, version, contact et réserves utiles. Les exemples doivent employer les ressources réellement livrées ou signaler explicitement leur statut d’illustration.

### Formats attendus et limites

- **HTML local** : navigation, structure sémantique, images et styles locaux, sans dépendance à un service pour la lecture. Les liens d’acquisition externes peuvent rester des liens clairement identifiés.
- **PDF de guide** : plusieurs pages, texte sélectionnable lorsque possible, sommaire, signets et liens, police et rendu vérifiés. Ne pas confondre le rendu PDF d’un logo avec un moteur de guide multipage.
- **Fiche de prise en main** : quelques pages adaptées au contenu, sans imposer artificiellement une page illisible.
- **Ressources et modèles** : uniquement les fichiers réellement produits, promis et autorisés.

Si un moteur PDF n’est pas disponible, annoncer cette limite dans le plan et fournir les sorties effectivement possibles. Aucun fichier vide ou HTML renommé en PDF. Ne pas annoncer une conformité d’accessibilité globale à partir de la seule présence de signets ou de contrastes satisfaisants.

### Organisation indicative de livraison

Étendre la convention de dossier déjà utilisée par l’application. Les noms ci-dessous sont proposés, pas des chemins garantis par l’existant :

```text
<dossier de livraison>/<marque>/<version>/
  README.txt
  manifest.json
  Guide/
    Brand-guide.pdf
    html/index.html
    html/resources/...
    Prise-en-main.pdf
  Logos/...
  Couleurs/...
  Modeles/...
  Exemples/...
  Licences-et-sources/...
  Controle/rapport-de-livraison.json
```

Éviter les dossiers vides qui font croire à des livrables. Les références internes utilisent des chemins relatifs transportables. Les rapports client ne doivent pas exposer les chemins personnels du poste, des secrets ou des notes internes non destinées à la remise.

Le manifeste doit identifier la version de projet, les fichiers effectivement produits, leur rôle, leur empreinte, leurs dépendances et les limites utiles. Conserver les informations internes sensibles dans le projet, hors de l’export client.

## 11. Plan de livraison, erreurs et sécurité des fichiers

Avant l’export, montrer les fichiers prévus, leurs formats, les dépendances, les blocages et les exclusions. Réutiliser le mécanisme de plan existant. Ne pas lancer la fabrication à chaque modification de texte ou à chaque ouverture du projet.

Autoriser un **brouillon explicitement signalé**, avec liste des éléments manquants et restrictions. Un brouillon n’autorise pas à transmettre une ressource sans droit établi. La mention « complet » et la remise finale exigent les contrôles applicables et la validation prévue.

Traiter les erreurs par sortie : expliquer ce qui a réussi et échoué, conserver les diagnostics utiles et ne pas compter une sortie en erreur comme livrée. Pour les livraisons finales, valider la cohérence du dossier avant son enregistrement comme version remise.

Préserver les garanties de l’export existant : écriture temporaire puis validation/renommage, absence d’écrasement des originaux et immutabilité des livraisons remises. Les nouvelles sorties doivent passer les mêmes contrôles de chemins, collisions de noms Windows, extensions, taille et écriture hors du dossier prévu. Vérifier les liens symboliques ou jonctions lorsque les entrées peuvent rediriger une écriture.

Une modification d’une dépendance marque les sorties concernées obsolètes avec une raison. Leur mise à jour demande une action de l’utilisateur et crée une nouvelle version si la précédente a été remise.

## 12. Intégration recommandée dans le code

Les noms suivants désignent des responsabilités proposées ; ils ne sont pas des fonctions déjà disponibles.

| Zone existante | Travail recommandé |
|---|---|
| `core/src/model.rs` et migration | Ajouter les structures, références et états, avec compatibilité des anciens projets. |
| Gestion de projet | Sauvegarde, validation des références, suivi des changements et sauvegardes de secours. |
| Module de validation à créer | Évaluer les exigences du périmètre et produire une liste de problèmes explicables. |
| Module de composition de guide à créer | Construire des pages depuis les mêmes données, sans inventer le contenu. |
| `core/src/export.rs` et sous-modules | Planifier, produire et inventorier les nouvelles sorties dans le mécanisme existant. |
| `kit/src/pages.rs` et pages associées | Édition des contenus, exemples liés, prévisualisation et correction guidée. |
| Oracle Assets / Oracle Font | Sélectionner les ressources et réutiliser leurs informations plutôt que refaire un catalogue. |
| CLI existante | Étendre les opérations de statut, plan et export si nécessaire ; documenter toute nouvelle commande seulement après son implémentation. |

Les commandes actuelles observées couvrent notamment création de projet, ajout de ressources, statut, plan, export et livraison. Ce cahier ne suppose pas l’existence d’une commande `guide` ou d’une validation complète de contenu.

## 13. Ordre de réalisation

### P0 — Données fiables et diagnostic

Périmètre, modules requis, état des contenus, références stables, droits, migrations et liste de problèmes. Résultat attendu : le logiciel sait expliquer ce qui manque sans détériorer les projets existants.

### P1 — Guide et kit de base réellement livrables

Édition des chapitres de base, règles mesurables, contrôles de couleurs et polices, composition HTML, moteur PDF si disponible, fiche de prise en main, plan, inventaire et livraison portable. Résultat attendu : une marque dans le périmètre de base peut être livrée et utilisée après recette.

### P2 — Extensions selon les usages

Modèles avancés, design system, mouvement, son, packaging, signalétique, données et internationalisation. Cet ordre est un ordre de développement : un module nécessaire au projet reste un blocage de complétude tant qu’il n’est pas pris en charge.

Ne pas appeler « complet » un jalon qui ne couvre qu’un sous-ensemble non annoncé du périmètre.

## 14. Critères de recette pour l’implémentation

Ces vérifications sont **à réaliser sur les futures fonctionnalités**. Elles n’ont pas été exécutées lors de la rédaction de ce document.

| ID | Scénario | Résultat à démontrer |
|---|---|---|
| R-01 | Ouvrir un projet v1 avec logos, couleurs, règles et livraisons | Données conservées, sauvegarde disponible, IDs stables, aucun ancien contenu auto-validé. |
| R-02 | Ouvrir une version de schéma inconnue ou des références invalides | Refus ou diagnostic sans réécriture destructrice. |
| R-03 | Préparer une marque sans voix, contact ou variante nécessaire | Manques visibles, sauvegarde possible, livraison complète bloquée selon le périmètre. |
| R-04 | Déclarer une rubrique non applicable | Justification et auteur conservés ; aucune conversion silencieuse des rubriques vides. |
| R-05 | Mesurer les trois paires Oracle et une valeur proche d’un seuil | Ratios conformes à la formule ; comparaison avant arrondi ; décision correcte selon taille et contexte. |
| R-06 | Employer du texte sur photo ou transparence | Absence de validation automatique injustifiée ; contrôle du rendu demandé. |
| R-07 | Fournir seulement un logo couleur ou un maître matriciel | Pas de variante inventée ni de faux vectoriel ; besoin de ressource expliqué. |
| R-08 | Retirer une police, un style ou un caractère nécessaire | Alerte précise, remplacement contrôlé et aucun fichier de fonte transmis par défaut. |
| R-09 | Ressource sans droit de transmission établi | Exclusion ou blocage correspondant ; le mode brouillon ne contourne pas la restriction. |
| R-10 | Préparer une livraison d’impression sans profil ou spécifications nécessaires | Pas de mention « prêt à imprimer » ; livraison numérique indépendante possible. |
| R-11 | Utiliser accents, textes longs et langues prévues, y compris RTL si demandé | Pas de texte coupé ni de police silencieusement absente ; contrôle des mises en page. |
| R-12 | Moteur PDF indisponible ou erreur de rendu | Erreur réelle, pas de faux PDF ni de succès déclaré pour le fichier manquant. |
| R-13 | Déplacer le dossier et ouvrir l’HTML hors ligne | Images, styles et liens internes fonctionnels ; limites des polices explicites. |
| R-14 | Livrer, modifier une ressource puis relivrer | Empreintes des maîtres et ancienne livraison conservées ; obsolescence détectée et nouvelle version. |
| R-15 | Retrouver un asset depuis les autres outils Oracle | Même ressource et collection ; aucune bibliothèque concurrente ni copie générale. |
| R-16 | Essayer un modèle dans une application de destination annoncée | Version, ouverture et possibilités d’édition réellement testées sur documents dédiés. |
| R-17 | Utiliser collisions de noms, chemins sortants et écritures interrompues | Aucun maître ou dossier extérieur altéré ; livraison défaillante non marquée comme remise. |
| R-18 | Confier trois supports représentatifs à une autre personne | Supports réalisés avec les seules règles et ressources livrées ; ambiguïtés restantes consignées. |

Adapter les tests au comportement réel : logique et migrations pour le cœur, fichiers générés pour les exports, contrôles visuels et essais de destination pour ce qu’un test automatisé ne démontre pas.

## 15. Preuves nécessaires pour annoncer la livraison du logiciel

Présenter séparément : code modifié, vérifications exécutées et résultats, application compilée, version portable produite, installation réellement mise à jour et essai dans l’interface. Une compilation réussie ne prouve pas à elle seule que l’application installée utilise les changements.

Associer chaque annonce à une preuve : projet de test, guide produit, dossier de kit, rapport de contrôle et réserves. Les données de démonstration restent identifiées comme fictives. Ne pas mélanger ces preuves futures avec la présente livraison, qui fournit la recherche et les instructions.

## 16. Référence de contenu et maintenance du cahier

La [recherche détaillée](RECHERCHE_BRAND_GUIDE.md) constitue la référence de contenu, avec ses sources officielles et la distinction entre recommandations, contraintes et critères d’accessibilité. Mettre à jour ces instructions si le modèle, la politique de licences ou les capacités d’export évoluent.

La définition de réussite reste concrète : une personne peut comprendre la marque, choisir les bonnes ressources, produire les supports prévus et maintenir le système sans dépendre de décisions absentes du guide.
