# Brand guide et évolution d’Oracle Brand Kit

Établi le **27 octobre 2026**. Ces documents répondent à deux besoins : savoir ce qu’une identité de marque doit documenter et donner à Oracle Brand Kit des instructions pour produire une livraison cohérente et utilisable.

## Documents et logiciel

1. [Recherche approfondie : contenu et livrables d’un brand guide](RECHERCHE_BRAND_GUIDE.md). Référentiel pour le designer : stratégie, identité visuelle et verbale, applications, accessibilité, production, droits et maintenance. Sources officielles commentées à la fin.
2. [Instructions pour Oracle Brand Kit](INSTRUCTIONS_ORACLE_BRAND_KIT.md). Cahier de spécifications : réutilisation de l’application actuelle, données à structurer, parcours, exports, contrôles et critères de recette.
3. [Étude d’Akrivi](ETUDE_AKRIVI.md). Analyse fonctionnelle de ses six outils à partir des sources publiques, comparaison et priorités pour Oracle.
4. [État de l’implémentation et vérifications](ETAT_IMPLEMENTATION.md). Fonctions disponibles, démarrage, preuves, limites et travail restant.
5. [Conception et utilisation du Studio](CONCEPTION_STUDIO.md). Grilles, présentations de pistes, Bento, exports et espace client hors ligne autour du même projet.

## Comment les utiliser

- Pour préparer une marque : définir son périmètre dans la recherche, puis réunir les contenus, fichiers et validations correspondants.
- Pour développer le logiciel : suivre les instructions produit, en commençant par le modèle de données et les contrôles de complétude.
- Pour livrer : vérifier séparément le guide explicatif, les fichiers utilisables, les droits et les essais de production.

Un **brand kit** fournit les ressources de travail. Un **brand guide** explique comment et pourquoi les utiliser. Une livraison complète doit relier les deux.

## Version réalisée le 27 octobre 2026

Le logiciel examiné se trouve dans `C:\Users\33652\Oracle-suite\oracle-brand`. Le chemin historique `C:\Users\33652\Desktop\Oracle` n’était pas présent au contrôle.

L’application comporte désormais un onglet **Brand guide**, avec douze chapitres communs et huit modules conditionnels. Il réutilise les logos, couleurs, typographies et règles du projet ; il permet de documenter les décisions, renseigner les autorisations de remise, contrôler des contrastes et générer un dossier HTML avec ressources autorisées, fiche de référence et manifeste. Un brouillon est explicitement marqué ; la livraison validée reste bloquée si les contrôles échouent.

Une [démonstration fictive](../../outputs/brand-guide-demo-2026-10-27/brand.json) a été créée et exportée. Ses chapitres restent à relire et à valider : ce n’est pas une identité client livrable. Le cahier décrit encore des objectifs plus larges que cette première implémentation. Le [bilan](ETAT_IMPLEMENTATION.md) distingue précisément ce qui fonctionne de ce qui reste prévu.

Cette livraison concerne les sources et une compilation de développement. Elle ne constitue pas une mise à jour de toutes les applications installées.

L’onglet **Studio** ajoute des grilles de présentation réglables, des pistes argumentées et ordonnables, deux modèles Bento et un espace client hors ligne. Les planches sont exportées en SVG de composition avec aperçus PNG, en PNG et en HTML ; les ressources restent celles du projet. Voir les limites et la recette dans la conception du Studio.

Aucune identité client n’est inventée ici. Les couleurs `#D31C3B` et `#1B1A17` servent d’exemple technique lié à Oracle ; elles ne doivent pas être imposées aux marques des clients.
