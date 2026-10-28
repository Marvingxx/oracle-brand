# Akrivi : étude fonctionnelle pour Oracle

Recherche du **27 octobre 2026**. Objet : comprendre le parcours public d’Akrivi et en déduire des fonctions utiles à Oracle. Méthode : documentation officielle et pages produit ; aucun accès à son code, à ses modèles payants ou à une installation authentifiée. Il s’agit d’une analyse fonctionnelle, pas d’une rétro-ingénierie de binaires ni d’un essai de ses logiciels.

## Ce que les sources permettent d’établir

Le [site officiel](https://www.akrivi.studio/) présente six outils : Logo Grids, Logo Presentations, Brand Guidelines, Bento Grids, Logo Exports et Brand Hub. Le parcours annoncé part des créations du designer dans Illustrator. Les pages de Studio 27.0, Logo Exports et Brand Hub affichent une inscription anticipée et une date à annoncer au contrôle ; cela ne permet pas de conclure que personne ne dispose d’une autre version. Les promesses de gain de temps et de qualité sont des affirmations commerciales, pas des résultats mesurés ici.

Le [mode d’emploi de Brand Guidelines Generator](https://akrivi.helpscoutdocs.com/article/41-how-to-use-brand-guidelines-generator) décrit la sélection de groupes de logos, le nom de marque, les typographies et les couleurs HEX, puis une génération depuis un modèle dans Illustrator. « Apply Changes » met à jour le document ; la documentation précise que le profil reste fixé au modèle, sans conversion des couleurs. Elle indique Illustrator 2023 minimum et recommande 2024, tandis que l’accueil public annonce une plage plus large : la compatibilité exacte doit donc être testée par version. Ses conseils de fermeture d’autres applications ne sont pas appliqués dans cette mission, pour préserver les documents utilisateur.

Le [guide de modification du fichier AI](https://akrivi.helpscoutdocs.com/article/91-how-to-edit-your-brand-guidelines-ai-file) indique un document Illustrator éditable et une convention de fichier à conserver pour les mises à jour. C’est une piste pour une sortie modifiable ; cela ne prouve ni une synchronisation bidirectionnelle générale, ni la compatibilité de tous les logiciels de création.

La page [Logo Exports](https://www.akrivi.studio/logo-exports) annonce jusqu’à dix variantes, cinq traitements colorés et six formats, avec une organisation par variation, couleur et format. Elle mentionne des destinations locales ou cloud et la possibilité d’associer guide et polices. Ce sont des capacités annoncées : aucun export Akrivi n’a été produit ni comparé dans cette étude.

La page [Brand Hub](https://www.akrivi.studio/brand-hub) présente une remise au client par lien avec accès aux ressources et au guide, sans compte client. La publication et la gestion sont liées au compte du créateur. La page ne suffit pas à vérifier la restauration d’une ancienne version, le fonctionnement hors ligne ou les garanties d’accès à long terme.

## Comparaison et décisions pour Oracle

Les recommandations de cette section sont notre conception produit, déduite des usages décrits. Elles ne décrivent pas le fonctionnement interne d’Akrivi.

| Outil présenté | Besoin à satisfaire | Réponse Oracle et suite à prévoir |
|---|---|---|
| Logo Grids | Expliquer construction, alignement et protection | Chapitre Logos et règles existantes réutilisés. À développer : unités mesurables, repères liés au maître, schémas de construction ; ne pas inventer une grille après coup. |
| Logo Presentations | Présenter des pistes et obtenir une décision | Les chapitres peuvent lier des ressources. À développer : concepts distincts, argumentaire, applications comparables et décision de sélection ; aucun générateur de présentation multipiste livré ici. |
| Brand Guidelines | Assembler une identité en document cohérent | Réalisé : douze chapitres, modules, validations, guide HTML et fiche. À développer : compositions éditoriales choisies, exemples plus riches, PDF natif et documents de mise en page modifiables. |
| Bento Grids | Composer une présentation courte de l’identité | À développer : une planche adaptable réunissant signe, palette, typo et applications depuis les mêmes références. Aucun générateur Bento livré. |
| Logo Exports | Préparer des fichiers prévisibles et bien rangés | Le moteur existant et ses presets sont conservés. Le nouvel export du guide remet les maîtres autorisés et leurs aperçus. À développer : matrice explicite rôle × variante × format × usage, sans simuler de variantes absentes. |
| Brand Hub | Permettre au client de retrouver la bonne version | Réalisé : dossier local transportable, guide et ressources avec manifeste. À développer : publication, permissions, révisions et retrait. Aucun hébergement ni lien client permanent créé. |

## Consignes de conception à conserver

Une seule identité doit alimenter guide, planches, mockups et livraisons. Les liens reposent sur les identifiants des ressources, pas sur le nom d’un fichier de présentation. Les originaux restent la référence ; les copies dans un dossier livré sont des instantanés, pas une seconde bibliothèque à maintenir.

Le designer renseigne les décisions, sélectionne les usages et valide le contenu. Un texte présent n’est pas nécessairement une bonne stratégie. L’application doit distinguer contenu manquant, brouillon, validation en cours de validité et module non applicable justifié. Toute modification de l’identité doit rendre visibles les validations devenues obsolètes.

Avant une remise, afficher les fichiers attendus, les limites de formats, les ressources exclues et les points à revoir. Les couleurs d’impression nécessitent des valeurs et un profil fournis ; une formule RVB → CMJN arbitraire ne suffit pas. Les polices nécessitent une autorisation de redistribution distincte de leur simple présence sur l’ordinateur. La première version du guide remet leurs références et liens de licence, sans copier les fichiers de police.

## Prochaine étape réaliste de rétro-ingénierie fonctionnelle

Pour vérifier les comportements au-delà des pages publiques, il faut une version identifiée d’Akrivi et un accès autorisé à ses fonctions. Préparer alors une marque fictive commune, exécuter les mêmes opérations, enregistrer entrées/sorties, temps, arborescences, pertes de mise à jour et erreurs. Comparer le résultat avec Oracle sans copier ses modèles ni son code. Aucun de ces essais n’a été réalisé ici.

Priorités proposées : rendre la livraison actuelle agréable et fiable ; ajouter des planches de présentation et la matrice d’exports ; produire ensuite un PDF éditorial et étudier un portail client. Le choix d’une bibliothèque locale centrale permet de progresser sans imposer Illustrator ni un abonnement supplémentaire au parcours Oracle.
