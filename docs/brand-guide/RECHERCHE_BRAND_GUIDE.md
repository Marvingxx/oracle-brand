# Ce qu’un brand guide complet doit contenir

**Recherche et référentiel de conception — 27 octobre 2026**  
Destinataires : designers, responsables de marque, développeurs d’Oracle Brand Kit et personnes qui utiliseront les livrables.

Mise en application : consulter l’[état d’Oracle Brand Kit](ETAT_IMPLEMENTATION.md) et l’[étude fonctionnelle d’Akrivi](ETUDE_AKRIVI.md). Ce référentiel décrit le résultat attendu ; la présence d’un sujet ici ne signifie pas que le logiciel l’automatise déjà.

## 1. Objectif et niveau d’exigence

Un bon guide permet à une personne qui n’a pas créé la marque de produire un support cohérent, de choisir les bons fichiers et de résoudre les situations courantes sans dépendre d’une explication orale.

Il n’existe pas un nombre universel de pages ni une liste identique pour toutes les marques. La complétude dépend des usages annoncés : une identité éditoriale, un service numérique et un produit emballé ont des besoins différents. Le présent document propose une synthèse opérationnelle ; les recommandations de structure ne sont pas une norme obligatoire.

Les sources distinguent trois choses : des exemples officiels de systèmes de marque, des exigences d’accessibilité applicables à certains usages, et des contraintes de production ou de licence. Il faut conserver cette distinction dans Oracle.

| Document ou système | Fonction |
|---|---|
| Plateforme de marque | Définir le positionnement, les publics, la promesse et les messages. |
| Charte graphique | Expliquer le système visuel et ses règles. |
| Brand guide | Relier stratégie, expression visuelle et verbale, applications et gouvernance. |
| Brand kit | Fournir les fichiers et modèles effectivement utilisables. |
| Design system | Décrire, si nécessaire, des composants numériques, leurs états et leurs comportements. |

Les fondations, ressources et disciplines regroupées par [IBM Design Language](https://www.ibm.com/design/language/) montrent qu’une identité dépasse le seul logo. Le [GOV.UK Design System](https://design-system.service.gov.uk/) permet de distinguer styles, composants et parcours récurrents : cette couche n’est nécessaire que si le périmètre de la marque la demande.

## 2. Définir le périmètre avant de remplir le guide

Documenter le nom de la marque, son propriétaire, le responsable de validation, les publics du guide, les langues, les pays et les supports prévus. Préciser ce qui est inclus, ce qui est exclu et ce qui reste à décider.

Choisir des usages réels : publication sociale, présentation commerciale, devis, site, application, affiche, packaging, signalétique, etc. Pour chaque usage, identifier qui produit, qui valide, quel fichier est attendu et comment il sera utilisé.

Une règle doit être identifiable comme **obligatoire**, **recommandée** ou **illustrative**. Un exemple de mise en page ne devient pas automatiquement une règle. Une section absente doit rester visible comme manquante ; « non applicable » nécessite une raison.

## 3. Stratégie et plateforme de marque

### Contenus à réunir

- Raison d’être et mission : ce que la marque cherche à accomplir et pour qui.
- Vision, si elle existe et aide les décisions de communication.
- Publics prioritaires, besoins, contextes d’usage et problèmes résolus.
- Positionnement : catégorie, différence pertinente, alternative et bénéfice principal.
- Promesse et preuves vérifiables : caractéristiques, méthode, expérience ou résultats documentés.
- Valeurs et personnalité, traduites en comportements observables.
- Messages clés, proposition de valeur, présentation courte et présentation développée.
- Architecture de marque : société, marques, gammes, produits et relations entre eux, si nécessaire.

« Moderne », « premium » ou « authentique » ne suffisent pas. Il faut expliquer leurs conséquences : vocabulaire, choix d’images, densité d’information, qualité de fabrication, ton, etc. Le guide ne doit pas fabriquer de preuves, de témoignages ou d’engagements pour remplir une rubrique.

**Résultat attendu :** une synthèse de marque exploitable et des messages validés, avec leur périmètre et leurs sources lorsque nécessaire.

## 4. Système de logos et signes distinctifs

### Variantes et choix

Présenter les signatures réellement fournies : principale, horizontale, verticale, symbole, version compacte, monochrome, blanche, inversée, favicon et éventuelles signatures secondaires. Toutes ne sont pas obligatoires ; celles que les usages nécessitent doivent exister et être validées.

Chaque version doit être reliée à son fichier. Donner un arbre de choix simple : petit espace, fond sombre, impression monochrome, support horizontal, partenariat. Expliquer aussi les cas où aucun fichier disponible ne convient et où il faut demander une variante au designer.

### Règles mesurables

- Zone de protection définie par une unité de référence visible et une mesure applicable.
- Taille minimale à l’écran et en impression, distincte pour chaque variante concernée.
- Fonds autorisés, fonds photographiques et conditions de contraste.
- Alignement, proportions, emplacement et relation avec les autres éléments.
- Assemblages avec un partenaire, ordre, séparation et cas d’équivalence ou de hiérarchie.
- Interdits illustrés : étirement, recoloration non prévue, effets, découpe, orientation ou fond inadaptés.

Les valeurs doivent être choisies sur le logo réel et testées à sa taille d’usage. Un pourcentage de protection ou une taille minimale repris d’une autre marque ne constitue pas une validation. Les [règles officielles d’Oxford](https://www.ox.ac.uk/about/the-university/brand/guidelines) montrent comment documenter ces sujets, avec des mesures propres à leur identité.

**Résultat attendu :** planche des variantes, règles illustrées, tableau d’usage et fichiers correspondants. Une modification de logo doit rester une décision du designer.

## 5. Couleurs : apparence, rôle et reproduction

Pour chaque couleur, fournir un nom stable, un rôle, le code HEX et les valeurs RGB dans un espace explicite, généralement sRGB pour le numérique. Décrire les couleurs principales, secondaires, neutres, de fond et de texte, puis les couleurs fonctionnelles si des interfaces sont prévues.

Préciser les associations autorisées, les couleurs de texte sur chaque fond, les priorités et, si elles sont utiles, les proportions recommandées. Une couleur d’erreur ou de succès ne doit pas être confondue avec un simple accent décoratif.

Pour l’impression, documenter les valeurs CMYK **avec leur condition de sortie ou profil ICC**. Des références Pantone, RAL ou autres peuvent être nécessaires selon les supports, mais elles doivent provenir d’un choix ou d’une validation réelle. Une conversion mathématique depuis un HEX ne garantit pas une correspondance matérielle exacte.

La [documentation Adobe sur la gestion des couleurs à l’impression](https://helpx.adobe.com/indesign/desktop/print/color-output-and-separations/use-color-management-when-printing.html) explique la relation entre profil de sortie, matériel et papier. Faire valider les spécifications et, si nécessaire, une épreuve par le prestataire ; éviter les doubles conversions.

**Résultat attendu :** palette à rôles explicites, tableau de combinaisons, références de reproduction et limites connues. Le [système de couleurs IBM](https://www.ibm.com/design/language/color/) constitue un exemple d’organisation de palette, pas une palette à reprendre automatiquement.

## 6. Typographie et composition du texte

Identifier les familles exactes, leurs sources, versions utiles, styles, graisses et, pour les fontes variables, les axes autorisés. Distinguer titres, sous-titres, texte courant, légendes, chiffres et éventuels usages techniques.

Documenter tailles, interlignages, approches, longueur de ligne, alignements, capitales, italique et associations. Préciser le contexte de chaque valeur : impression, écran, présentation, petit format ; une seule échelle ne convient pas à tous les supports.

Tester les accents, signes monétaires, chiffres, ponctuation et langues prévues. Prévoir une solution de remplacement pour les destinataires qui ne disposent pas de la police et expliquer son effet sur la mise en page. Un remplacement n’est pas forcément métriquement équivalent.

Fournir les liens d’acquisition et les conditions d’usage connues : poste de travail, web, application, document incorporé, transmission éventuelle. Une autorisation d’utilisation ne signifie pas une autorisation de redistribution. [Adobe Fonts](https://helpx.adobe.com/fonts/web/font-licensing/font-licensing.html) distingue notamment les documents réalisés avec les polices de la transmission des fichiers de polices eux-mêmes.

**Résultat attendu :** spécimen, hiérarchie, règles d’association, test de caractères, polices de remplacement et notice de licence. Ne fournir les fichiers de fonte que si la politique du produit et les droits l’autorisent ; la politique actuelle d’Oracle prévoit de ne pas les livrer.

## 7. Photographie, illustration, icônes et autres éléments visuels

Une planche d’inspiration aide à discuter ; elle ne remplace pas des consignes reproductibles. Préciser les sujets, personnes, cadrages, lumière, palette, contraste, profondeur, retouche et exclusions. Ajouter des exemples autorisés et des contre-exemples expliqués.

Pour l’illustration et les icônes, décrire construction, grille, épaisseur des traits, angles, remplissages, niveau de détail, proportions et lisibilité aux petites tailles. Si la marque utilise motifs, textures ou 3D, préciser leur fabrication, leur rôle et leurs associations.

Documenter l’origine, l’auteur, les licences, les crédits et les éventuelles autorisations liées aux personnes ou aux lieux. Les informations non connues restent à vérifier ; la présence d’un fichier dans la bibliothèque ne prouve pas ses droits d’usage.

**Résultat attendu :** direction iconographique, bibliothèque sélectionnée et règles de sélection permettant d’étendre la marque sans copier les seuls exemples.

## 8. Mise en page et système de composition

Définir les grilles, marges, espacements, alignements, hiérarchie, rapports entre images et textes, densité et placement du logo. Expliquer les éléments fixes et les zones qui peuvent varier.

Montrer plusieurs rapports de format et des cas difficiles : titre long, peu d’images, contenu dense, portrait, paysage et petit écran. Les principes doivent pouvoir être adaptés ; un seul modèle figé ne suffit pas.

Pour une interface numérique, compléter par les états et comportements réellement nécessaires : survol, focus, sélection, erreur, chargement, désactivation et modes clair/sombre. Distinguer ces composants du guide graphique général.

**Résultat attendu :** règles de composition, quelques constructions annotées et modèles réutilisables associés aux usages retenus.

## 9. Identité verbale, voix et ton

Décrire la voix stable de la marque puis ses variations selon les contextes : découverte, vente, assistance, erreur, annonce délicate. Un même humour ne convient pas à toutes les situations. Le [guide de voix et de ton de Mailchimp](https://styleguide.mailchimp.com/voice-and-tone/) illustre cette distinction et la priorité donnée à la clarté.

Préciser tutoiement ou vouvoiement, niveau de langue, vocabulaire, expressions à éviter, place de l’humour et limites. Fournir un lexique pour les noms de produits, les abréviations et les termes métier, puis les conventions utiles de dates, unités et ponctuation.

Rédiger des exemples validés : présentation courte, biographie, titre, appel à l’action, message d’erreur et réponse de service. Ajouter des réécritures avant/après qui expliquent le choix. Prévoir des adaptations culturelles plutôt qu’une traduction mécanique quand le périmètre l’exige.

**Résultat attendu :** règles éditoriales utilisables par une personne qui ne connaît pas encore la marque.

## 10. Accessibilité et lisibilité

### Contraste mesurable

Pour les contenus numériques visés par WCAG 2.2 niveau AA, le texte courant nécessite généralement un contraste d’au moins **4,5:1**. Le grand texte exige **3:1** : au moins 18 points en graisse normale, ou 14 points en gras, soit environ 24 et 18,67 pixels CSS. Les logotypes disposent d’une exception ; elle ne s’étend pas aux autres textes de marque. Voir [WCAG : contraste minimal](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html).

Les informations visuelles nécessaires à l’identification d’un composant ou à la compréhension d’un graphique relèvent également de critères de contraste, avec des exceptions spécifiques. Ne pas transformer cela en obligation uniforme pour tous les éléments décoratifs. Voir [WCAG : contraste des éléments non textuels](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html).

Exemple calculé pour les couleurs Oracle, **opaques et en sRGB**, sur fonds unis :

| Premier plan / fond | Ratio approximatif | Lecture du résultat |
|---|---:|---|
| `#D31C3B` / `#1B1A17` | 3,31:1 | Insuffisant pour du texte courant AA ; seuil de grand texte atteint. |
| `#FFFFFF` / `#D31C3B` | 5,25:1 | Seuil de texte courant AA atteint. |
| `#FFFFFF` / `#1B1A17` | 17,40:1 | Seuil de texte courant AA atteint. |

Comparer les valeurs non arrondies aux seuils. Une transparence, un dégradé ou une photographie nécessite une vérification du rendu réel ; le ratio de deux codes seuls ne valide pas l’image composée.

### Au-delà du contraste

Ne pas transmettre une information uniquement par la couleur : ajouter texte, forme, motif ou autre indication pertinente. Voir [WCAG : utilisation de la couleur](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html).

Selon les supports, prévoir alternatives textuelles, sous-titres, ordre de lecture, structure de titres, navigation clavier, focus visible, réduction des animations et textes compréhensibles. Vérifier les documents exportés, pas seulement la palette.

**Résultat attendu :** associations validées pour des usages précis, règles d’accessibilité applicables et contrôles effectués. Un tableau de contraste ne démontre pas la conformité WCAG globale d’un site ou d’un PDF.

## 11. Applications et modèles utilisables

Sélectionner les supports prioritaires au début du projet puis fournir, pour chacun, un exemple abouti et un modèle réellement modifiable si c’est le livrable promis. Une image de mockup ne remplace pas le fichier de production.

Les modèles doivent indiquer les zones fixes, zones modifiables, polices nécessaires, images remplaçables, contraintes de texte et modalités d’export. Prévoir au moins un essai avec un titre long et un contenu différent de la démonstration.

Les [applications de marque d’Oxford](https://www.ox.ac.uk/about/the-university/brand/applying) montrent comment prolonger des règles par des modèles de supports concrets. Leur catalogue est un exemple d’approche, pas une liste de livrables obligatoires pour chaque client.

**Résultat attendu :** un ensemble adapté aux usages du client : publications, présentation, document commercial, signature, affiche, interface ou autres supports contractualisés. Documenter le logiciel, sa version testée et les limites d’édition connues.

## 12. Production, fichiers et remise du kit

Préserver les fichiers maîtres. Identifier les exports par rôle et usage, avec un nom compréhensible. Vérifier les dimensions, la transparence, le fond, la résolution utile et la nature réellement vectorielle ou matricielle du contenu.

| Besoin | Fichiers possibles | Point à vérifier |
|---|---|---|
| Logo vectoriel | SVG, PDF adapté | Le contenu doit réellement être vectoriel ; un PDF peut contenir une image. |
| Image transparente | PNG, éventuellement WebP | Transparence, dimensions et prise en charge par la destination. |
| Image photographique | JPEG, WebP, PNG selon le cas | Compression, fond, profil et qualité visuelle. |
| Impression | PDF selon les spécifications du prestataire | Format, fonds perdus, profil, polices et contenu effectivement demandé. |
| Modèle modifiable | Format natif ou format d’échange convenu | Ouverture et édition testées dans le logiciel annoncé. |
| Guide | PDF et/ou HTML | Lecture, navigation, liens, texte et portabilité. |

Ne pas annoncer « compatible Photoshop/Affinity » à partir d’une extension seulement. Ouvrir un fichier et conserver ses fonctions d’édition sont deux vérifications différentes. Les formats proposés ici restent dépendants des capacités réellement implémentées dans Oracle et des essais de destination.

Ne pas imposer systématiquement « 300 dpi » ou « 3 mm de fond perdu » : les valeurs dépendent du support et de la fabrication. Faire préciser ces contraintes au prestataire concerné.

**Résultat attendu :** dossier organisé avec notice de démarrage, inventaire, fichiers utilisables, licences ou liens de licence, versions et limites. La bibliothèque de travail peut rester centrale ; la livraison est une copie volontaire des seuls éléments nécessaires et autorisés.

## 13. Gouvernance et maintenance

Inscrire la version, la date d’entrée en vigueur, le responsable, le contact et l’emplacement de la référence officielle. Définir qui peut changer une règle et comment les exceptions sont approuvées.

Maintenir un historique des modifications, les versions retirées, leurs remplacements et les raisons d’obsolescence. Distinguer un contenu en préparation, un contenu validé et une livraison effectivement remise. Conserver l’état exact d’une livraison passée.

Documenter la provenance et les droits des ressources, les territoires ou durées lorsqu’ils sont pertinents, les crédits et les contraintes de diffusion. Une mention générique « tous droits réservés » ne résout pas les licences des ressources tierces. Les situations juridiques particulières restent à faire valider par une personne compétente.

**Résultat attendu :** une marque que l’on peut maintenir, avec une responsabilité identifiable et des versions traçables.

## 14. Modules conditionnels : ne pas les oublier lorsqu’ils sont pertinents

| Module | Quand l’inclure | Contenu à préciser |
|---|---|---|
| Architecture et partenariat | Plusieurs marques, gammes ou partenaires | Hiérarchie, signatures, relation entre noms et règles d’association. |
| Produit numérique | Site ou application dans le périmètre | Composants, états, interactions, accessibilité et jetons de design si utiles. |
| Mouvement | Vidéos, animations ou interfaces animées | Rythme, transitions, durées, courbes, titres et réduction des mouvements. |
| Son et prononciation | Identité sonore, voix ou noms ambigus | Signature, usages, prononciation, voix, mixage et droits. |
| Packaging | Produits emballés | Gabarit, matériaux, finitions, contraintes du fabricant et mentions à valider. |
| Signalétique | Lieux physiques | Distance de lecture, échelle, contraste, pose, matériaux et orientation. |
| Données et graphiques | Rapports, infographies ou tableaux de bord | Échelles, légendes, unités, annotations et distinctions sans couleur seule. |
| Internationalisation | Plusieurs langues ou territoires | Traductions, caractères, textes longs, sens de lecture et variantes locales. |
| Mascotte ou personnage | Personnage récurrent | Construction, proportions, poses, expressions, usages interdits et droits. |
| Usage d’IA dans la production client | Un tel usage est réellement prévu | Provenance, validation humaine et règles de divulgation définies par le client. Cela n’ajoute pas d’IA à Oracle. |

Un module conditionnel devient nécessaire dès qu’un usage validé en dépend. « Facultatif » ne doit pas servir à ignorer une exigence du projet.

## 15. Livrables qui rendent le système exploitable

1. **Guide complet** : explications, règles, exemples, limites, navigation et liens vers les ressources.
2. **Fiche de prise en main** : décisions courantes, palette utile, typographie, choix de logo et contact.
3. **Kit de ressources** : originaux de référence et exports autorisés, rangés et identifiés sans ambiguïté.
4. **Modèles modifiables** : uniquement ceux promis dans le périmètre, accompagnés de leurs dépendances.
5. **Dossier de maintenance** : notice, inventaire, droits, versions, contrôles et procédure de mise à jour.

Ces livrables peuvent être regroupés ; leur fonction doit rester identifiable. Un guide esthétique sans ressources ni instructions de production n’est pas un kit opérationnel.

## 16. Recette : comment juger qu’il est complet

- Le périmètre et les exclusions ont été validés ; chaque rubrique requise est traitée.
- Les règles renvoient à des fichiers présents et les exemples correspondent aux variantes fournies.
- Une autre personne réussit trois supports représentatifs, choisis dans le périmètre, sans devoir inventer des règles structurantes.
- Les logos ont été vérifiés aux petites tailles, sur les fonds nécessaires et en monochrome si cet usage est prévu.
- Les caractères et polices nécessaires ont été vérifiés ; l’absence d’une police conduit à une solution expliquée.
- Les associations de couleurs sont évaluées pour leur usage ; les cas non mesurables automatiquement sont signalés.
- Les modèles supportent un contenu différent de celui de la démonstration.
- Les fichiers promis s’ouvrent et, si promis, s’éditent dans les logiciels de destination testés.
- Les contraintes d’impression ont été confirmées si l’impression fait partie du périmètre.
- Les droits de transmission sont établis et les fichiers non autorisés sont exclus.
- La notice, les liens et les ressources fonctionnent après déplacement du dossier de livraison.
- Les éléments non validés restent explicitement identifiés ; aucun score global ne masque un blocage.

La qualité se juge sur ces usages et ces preuves, pas sur le nombre de pages.

## 17. Sources et méthode

Sources primaires consultées le **27 octobre 2026**. Les propositions de structure et de produit formulées ici sont une synthèse ; elles ne sont pas attribuées en bloc à l’un de ces organismes. Les liens de référence peuvent évoluer après cette date.

| Source officielle | Apport précis à la recherche |
|---|---|
| [IBM Design Language](https://www.ibm.com/design/language/) | Étendue des disciplines et ressources d’une identité. |
| [IBM — Color](https://www.ibm.com/design/language/color/) | Organisation et usages d’un système de couleurs. |
| [Oxford — Guidelines](https://www.ox.ac.uk/about/the-university/brand/guidelines) | Documentation d’un logo, de ses règles et de son environnement visuel. |
| [Oxford — Applying the brand](https://www.ox.ac.uk/about/the-university/brand/applying) | Passage des principes aux modèles concrets. |
| [Mailchimp — Voice and Tone](https://styleguide.mailchimp.com/voice-and-tone/) | Distinction entre voix de marque et ton selon le contexte. |
| [GOV.UK Design System](https://design-system.service.gov.uk/) | Distinction entre styles, composants et modèles d’interaction. |
| [W3C — Contrast Minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) | Seuils de contraste du texte, grandes tailles et exceptions. |
| [W3C — Non-text Contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html) | Contraste des éléments visuels utiles aux interfaces et graphiques. |
| [W3C — Use of Color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html) | Information ne reposant pas exclusivement sur la couleur. |
| [Adobe Fonts — Font licensing](https://helpx.adobe.com/fonts/web/font-licensing/font-licensing.html) | Différence entre utilisation d’une police, documents produits et transmission des fontes. |
| [Adobe InDesign — Color management when printing](https://helpx.adobe.com/indesign/desktop/print/color-output-and-separations/use-color-management-when-printing.html) | Profils et conditions de sortie en impression. |

Pour traduire ce référentiel en logiciel, lire les [instructions Oracle Brand Kit](INSTRUCTIONS_ORACLE_BRAND_KIT.md).
