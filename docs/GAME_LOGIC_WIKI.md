# Logique de jeu — synthèse du wiki

Ce document rassemble les règles de référence utiles à Ante et sert de point d'entrée vers les fiches de la [roadmap fonctionnelle](ROADMAP_FEATURES.md). La cible de comparaison actuelle est Balatro 1.0.1o, conformément au guide d'interface. Cette synthèse est volontairement orientée implémentation : elle ne remplace pas les pages détaillées du wiki.

## Boucle d'une run

Une run progresse par antes et blinds. La petite blind, la grande blind et la blind Boss ont chacune un objectif de score; gagner une blind ouvre l'encaissement et la boutique, puis la prochaine étape. La boutique permet d'acheter ou vendre, d'utiliser des consommables, de relancer les offres et d'ouvrir des boosters. Les packs proposent un nombre limité de choix, parfois suivis du ciblage de cartes. Une run se termine par une victoire ou une défaite, avec poursuite éventuelle en Endless.

## Main, cartes et score

Les cartes sélectionnées déterminent une main de poker selon la priorité des combinaisons reconnues. Les cartes qui contribuent au score sont résolues de gauche à droite. Les effets propres aux cartes et les effets Joker se déclenchent pendant cette résolution; la position des cartes et des Jokers peut donc changer le résultat. Le score est construit par additions de Chips et Mult puis multiplications, dans l'ordre d'activation. Les cartes en main peuvent aussi déclencher des effets distincts.

Le tri automatique par rang et par couleur existe déjà pour la main. Il ne remplace pas le déplacement manuel. Le tri manuel de la main doit modifier l'ordre visuel utilisé lors de la résolution, sans modifier les identifiants des cartes ni consommer du hasard.

## Jokers et ordre d'activation

Les Jokers déclenchent leurs effets dans l'ordre gauche-à-droite de leur zone, au sein de chaque fenêtre d'activation. Les effets qui se déclenchent à des moments différents conservent leurs fenêtres propres; déplacer un Joker ne change pas leur priorité de phase, mais ordonne les effets qui s'exécutent dans la même fenêtre. Les effets de répétition peuvent relancer des activations de cartes.

Le client doit permettre de réordonner les Jokers détenus. La simulation reste seule propriétaire de l'ordre et la trace de score doit refléter cet ordre. Le déplacement doit préserver l'objet et ses états (édition, autocollants, valeurs cumulées).

## Valeurs courantes des Jokers

L'infobulle d'un Joker qui évolue se termine par une ligne grise « (Currently …) ». La valeur y est colorée selon son effet : +Mult en rouge, +Chips en bleu, XMult dans une pastille rouge, argent en jaune, compteur en orange. Dans l'original, cette ligne existe pour Abstract Joker, Bootstraps, Ceremonial Dagger, Erosion, Flash Card, Green Joker, Red Card, Ride the Bus, Spare Trousers et Swashbuckler (+Mult). Elle existe aussi pour Blue Joker, Bull, Castle, Runner, Square Joker, Stone Joker et Wee Joker (+Chips). Côté XMult : Canio, Campfire, Constellation, Glass Joker, Hit the Road, Hologram, Lucky Cat, Madness, Obelisk, Throwback, Vampire, Steel Joker, Yorick et Joker Stencil. Côté argent : Cloud 9 et Satellite. Fortune Teller, Driver's License et Invisible Joker affichent un compteur. Ice Cream, Popcorn, Ramen et Turtle Bean montrent leur valeur dans la ligne principale.

Ante calcule la valeur côté simulation (`CurrentValue`) et la met en forme côté client. Comme dans l'original, Green Joker, Castle, Obelisk et Yorick gardent leur valeur sur la carte elle-même : Green Joker et Castle la conservent toute la run, Obelisk revient à X1 quand la main jouée est strictement la plus jouée, Yorick compte les cartes défaussées depuis son acquisition. Joker Stencil vaut X(cases vides + Stencils détenus) et ne s'applique que s'il reste une case vide ; Hit the Road repart à X1 à chaque manche. Seuls les Jokers implémentés par le moteur sont couverts ; les écarts de règle du moteur sont suivis dans [la roadmap des fonctionnalités](ROADMAP_FEATURES.md).

## Rythme du score

Les délais de l'original sont exprimés en secondes de jeu. L'horloge avance `vitesse` fois plus vite que le temps réel, et s'accélère encore sur les longues séquences. Pour une main de cinq cartes qui ne donnent que des chips, sans Joker, la séquence dure environ 8,75 s à 1x. Le détail des délais est dans [le guide legacy](UI_LEGACY_GUIDE.md#rythme-du-score-et-vitesse).

## Améliorations, éditions et sceaux

Les améliorations modifient le comportement de la carte (par exemple Chips, Mult, verre, acier, pierre, or ou chance). Les éditions ajoutent leurs propres bonus. Les sceaux sont un modificateur séparé :

| Sceau | Règle résumée |
| --- | --- |
| Or | Donne de l'argent lorsque la carte est marquée. |
| Rouge | Répète l'activation de la carte marquée. |
| Bleu | Peut créer une carte Planète à la fin de la manche si elle reste en main. |
| Violet | Crée une carte Tarot lorsque la carte marquée est défaussée, si une place est disponible. |

Le moteur expérimental transporte déjà le sceau dans la projection et affiche un résumé d'effet. La couverture d'attribution et les déclenchements réels doivent être vérifiés séparément avant de considérer la mécanique terminée.

## Débuff

Une carte débuffée conserve son identité et son emplacement, mais ses effets affectés ne contribuent pas comme une carte active. Le client actuel applique une désaturation grise à certains rendus de cartes. Le rendu de référence à reproduire ajoute des flèches rouges marquées et une teinte rouge subtile. Les cartes dans le deck, la main, les packs et les autres surfaces doivent partager le même état visuel lorsqu'elles affichent une carte débuffée.

## Présentation et interaction

Le survol d'une carte présente son identité, sa description et ses modificateurs. Une carte de pack doit donc montrer la description de l'objet choisi; l'instruction « Choose this card from the pack » appartient au contexte de choix, pas à l'infobulle de la carte. Le mouvement de survol et l'élévation de sélection doivent être communs aux types de cartes interactifs et aux rendus du deck.

## Sources et vérification

- [Balatro Wiki](https://balatrowiki.org/) — source principale demandée; la consultation directe automatisée est bloquée par robots.txt. Les faits ci-dessus sont à valider contre le jeu de référence et le guide local.
- [Guide de séquence d'activation](https://balatrogame.fandom.com/wiki/Guide%3A_Activation_Sequence) — ordre de résolution et répétitions.
- [Types d'activation Joker](https://balatrogame.fandom.com/wiki/Activation_Type) — fenêtres et ordre des effets.
- [Guide d'interface legacy d'Ante](UI_LEGACY_GUIDE.md) — observations locales de Balatro 1.0.1o et points d'entrée.
- [Guide d'interface Bevy d'Ante](UI_BEVY_GUIDE.md) — correspondance vers le client et écarts connus.

Cette synthèse ne constitue pas une autorisation de recopier le code, les textes ou les ressources du jeu. Implémenter les règles indépendamment et relever explicitement les divergences encore non vérifiées.
