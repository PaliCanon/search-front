//! The footer pages in French: generated from today's site's
//! `web/src/locales/fr.json` `pages.*` (verbatim; `{{publisher}}`,
//! `{{contact}}` and `{{date}}` resolved; the Legal page and the two
//! pointers to it dropped — task 001, fix round 4; AI mentions removed —
//! task 004). Data, not code: regenerate rather than edit by hand.

use super::{AppBlock, Cta, InstagramBlock, PageText, Pages, SearchBlock, Section, Table};

/// Every footer page, in this language.
pub(super) const PAGES: Pages = Pages {
    acknowledgements: PageText {
        title: "Remerciements",
        description: "Crédits et licences des suttas, traductions et dictionnaire dans lesquels PaliCanon cherche.",
        intro: &[
            "PaliCanon est un moteur de recherche, pas une source. Tout ce qu’il vous permet de trouver a été réalisé par d’autres et offert ; cette page dit par qui, et sous quelle licence.",
        ],
        search: None,
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "Les suttas",
                p: &[
                    "Les traductions anglaises sont de **Bhikkhu Sujato**, qui a traduit les quatre principaux Nikāyas et une grande partie du Khuddaka, et a versé l’ensemble dans le domaine public. Le texte pāli est l’édition **Mahāsaṅgīti Tipiṭaka** préparée par la Dhamma Society. Tous deux sont publiés par [SuttaCentral](https://suttacentral.net) sous la dédicace Creative Commons CC0, avec les titres des suttas, leurs résumés et le système de références que suit PaliCanon. Chaque résultat renvoie à SuttaCentral ; lisez-y les suttas en entier, et pensez à [les soutenir](https://suttacentral.net/donations).",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "Traductions françaises et vietnamiennes",
                p: &[
                    "Quand PaliCanon est utilisé en français ou en vietnamien, les titres des suttas viennent des traductions publiées par [SuttaCentral](https://suttacentral.net) (chaque traduction reste celle de son traducteur), lorsqu’un sutta en a une ; la recherche porte aussi sur elles : une question posée en français ou en vietnamien trouve ses réponses dans ces textes. Les textes français (1 184 suttas) sont de **Môhan Wijayaratna**, **Rémy Zins**, **Claude et Chandhana Le Ninan**, **Christelle Ah-Mouck**, **Marion Ah-Mouck**, **Fernand Hû**, **U Ko Lay**, **U Tin U (Myaung)**, **Florent Robert**, **Indavati**, **Régis Xhardé**, et des équipes de **buddha-vacana.org** et **Canonpali.org**. Les textes vietnamiens (4 240 suttas) sont de **Thích Minh Châu**, **Bhikkhu Indacanda** et **Phan Tuấn Anh**. Tous ont été recueillis sur [SuttaCentral](https://suttacentral.net) et sont reproduits sans modification ; ils appartiennent à leurs traducteurs.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "Le dictionnaire",
                p: &[
                    "Le sens des mots pālis provient du **Digital Pāḷi Dictionary** de Bhikkhu Bodhirasa, [digitalpalidictionary.github.io](https://digitalpalidictionary.github.io), sous licence [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/). PaliCanon est non commercial et reproduit les entrées sans modification ; toute autre utilisation des données du dictionnaire doit respecter cette licence.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "Avec gratitude",
                p: &[
                    "À Bhikkhu Sujato, pour avoir traduit le Sutta Piṭaka et l’avoir offert à tous ; à Bhikkhu Bodhirasa, pour le dictionnaire ; à l’équipe de SuttaCentral, pour la publication et l’entretien des textes ; et au projet Mahāsaṅgīti de la Dhamma Society, pour l’édition pālie.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    terms: PageText {
        title: "Conditions d’utilisation",
        description: "Les conditions régissant l’utilisation de PaliCanon.",
        intro: &[
            "Cette traduction est fournie pour votre confort ; en cas de divergence, la version anglaise prévaut.",
            "Les présentes conditions régissent votre utilisation de PaliCanon (le « Service »), édité par Thibault Rivière (l’« Éditeur »). En utilisant le Service, vous les acceptez.",
        ],
        search: None,
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "1. Le Service",
                p: &[
                    "Le Service vous permet de chercher dans les suttas du Sutta Piṭaka par mots exacts et par le sens, de lire les passages trouvés avec un lien vers leur source et de consulter le sens des mots pālis. Il est fourni gratuitement et sans inscription.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "2. Acceptation",
                p: &[
                    "Utiliser le Service vaut acceptation des présentes conditions. Vous pouvez cesser d’utiliser le Service à tout moment ; les données stockées dans votre navigateur vous appartiennent et vous pouvez les supprimer (voir [Confidentialité](/privacy)).",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "3. Textes et propriété intellectuelle",
                p: &[
                    "Les suttas sont dans le domaine public et le dictionnaire est sous licence CC BY-NC 4.0, comme détaillé dans les [Remerciements](/acknowledgements). Le Service lui-même — son moteur de recherche, son interface, ses index et son nom — est la propriété de l’Éditeur. Vous pouvez l’utiliser pour une consultation personnelle et non commerciale. L’extraction automatisée en masse des résultats, la reproduction du Service ou l’utilisation de son API en dehors du site public nécessitent l’autorisation écrite préalable de l’Éditeur.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "4. Disponibilité et responsabilité",
                p: &[
                    "Le Service est fourni « en l’état », gratuitement. L’Éditeur fait de son mieux pour qu’il soit exact et disponible mais n’offre aucune garantie d’aucune sorte, et peut le modifier, le suspendre ou l’interrompre à tout moment. Dans les limites permises par la loi, l’Éditeur n’est pas responsable des pertes résultant de l’utilisation du Service, de son indisponibilité, ou du contenu des sites tiers vers lesquels il renvoie. Rien ici ne limite une responsabilité que la loi ne permet pas de limiter.",
                    "Vous vous engagez à ne pas perturber le Service, à ne pas envoyer de trafic abusif ou automatisé, et à ne pas tenter d’atteindre d’autres systèmes que le site public. L’Éditeur peut restreindre l’accès de toute source qui le ferait.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "5. Droit applicable et modifications",
                p: &[
                    "Les présentes conditions sont régies par le droit du lieu de résidence de l’Éditeur, sans préjudice des protections impératives accordées aux consommateurs par votre propre pays. L’Éditeur peut les mettre à jour ; la date ci-dessous est celle de la version en vigueur, et la poursuite de l’utilisation après une modification vaut acceptation de celle-ci.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    privacy: PageText {
        title: "Confidentialité",
        description: "La politique de confidentialité RGPD de PaliCanon : quelles données sont traitées, pourquoi, et pour combien de temps.",
        intro: &[
            "Cette traduction est fournie pour votre confort ; en cas de divergence, la version anglaise prévaut.",
            "Voici l’information requise par le Règlement général sur la protection des données (RGPD). Le responsable du traitement est Thibault Rivière, contact riviere.thibault@icloud.com. PaliCanon n’a pas de comptes, ne dépose aucun cookie et n’utilise ni publicité ni pistage par des tiers.",
        ],
        search: None,
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "1. Ce qui est traité, pourquoi, et sur quelle base",
                p: &[
                    "Merci de ne saisir aucune information personnelle — la vôtre ou celle de quiconque — dans le champ de recherche : les requêtes sont conservées, et elles ne restent anonymes que parce que rien d’identifiant n’y est associé.",
                ],
                list: &[],
                table: Some(Table {
                    head: &[
                        "Données",
                        "Finalité",
                        "Base légale (RGPD art. 6)",
                        "Conservation",
                    ],
                    rows: &[
                        &[
                            "Requêtes de recherche : le texte saisi, le pays d’origine, l’heure (arrondie à l’heure), et le déroulement de la recherche (nombre de résultats, correction appliquée, latence). Aucune adresse IP ni aucun identifiant n’y est associé.",
                            "Améliorer la recherche : voir ce que cherchent les lecteurs, ce qui ne donne rien, quelles orthographes et quels termes pālis sont utilisés.",
                            "Intérêt légitime (6.1.f) : faire fonctionner et améliorer un service public gratuit, avec des données qui ne peuvent être rattachées à une personne.",
                            "Jusqu’à 24 mois ; statistiques agrégées sans limite de durée.",
                        ],
                        &[
                            "Journaux de connexion tenus par les hébergeurs : adresse IP, heure, URL demandée, user agent.",
                            "Sécurité, prévention des abus, maintien du Service en fonctionnement.",
                            "Intérêt légitime (6.1.f).",
                            "Par les hébergeurs, selon leurs propres politiques ; en général 30 jours ou moins.",
                        ],
                    ],
                }),
                cta: None,
            },
            Section {
                h: "2. Destinataires et transferts",
                p: &[
                    "Le site est servi par Cloudflare, Inc. (États-Unis) et l’API de recherche par Google Cloud (Google Ireland Limited ; serveurs situés dans l’UE ou aux États-Unis). Tous deux agissent en tant que sous-traitants selon leurs conditions standard de traitement des données, qui incluent les clauses contractuelles types de la Commission européenne pour les transferts hors de l’UE. Aucune donnée n’est vendue ni partagée avec qui que ce soit d’autre.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "3. Ce qui reste dans votre navigateur",
                p: &[
                    "Votre thème et votre langue sont stockés par votre navigateur (localStorage) et ne sont jamais envoyés au Service. Ce stockage est strictement nécessaire pour fournir les fonctions que vous avez demandées et ne requiert donc aucun bandeau de consentement. Vous pouvez l’effacer à tout moment en effaçant les données du site dans votre navigateur.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "4. Vos droits",
                p: &[
                    "Vous disposez des droits d’accès, de rectification, d’effacement, de limitation, de portabilité et d’opposition prévus par le RGPD, ainsi que du droit d’introduire une réclamation auprès d’une autorité de contrôle (en France, la CNIL). Écrivez à riviere.thibault@icloud.com. Notez que les requêtes de recherche sont conservées sans aucun identifiant : le Service ne peut pas retrouver « vos » requêtes, et ne peut donc ni vous les communiquer ni les supprimer individuellement — c’est d’ailleurs ce qui les garde anonymes.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    support: PageText {
        title: "Soutenir",
        description: "Ce que coûte le fonctionnement de PaliCanon et de l’app Tipiṭaka, et comment vous pouvez aider à les garder gratuits.",
        intro: &[
            "PaliCanon et l’application Tipiṭaka pour iPhone sont gratuites : pas de publicité, pas de compte, rien de vous à vendre. Une seule personne les conçoit et les fait vivre.",
        ],
        search: Some(SearchBlock {
            h: "La recherche",
            tagline: "Cherchez dans tout le Canon pāli par les mots dont vous vous souvenez ou par ce que vous voulez dire : en anglais, en français et en vietnamien, avec le passage et un lien pour le lire.",
            cta: "Chercher dans le Canon",
            alts: &[
                "L’accueil de la recherche",
                "Résultats pour une question",
                "Résultats par mots exacts",
                "Une recherche en français",
                "Le thème sombre",
            ],
        }),
        app: Some(AppBlock {
            h: "L’application Tipiṭaka",
            tagline: "Tout le Canon pali sur votre iPhone, hors ligne : pali et traductions côte à côte, un dictionnaire sur chaque mot, des parcours d’étude, des notes — en anglais, en français et en vietnamien.",
            cta: "Télécharger l’app iPhone",
            alts: &[
                "Écran d’accueil de l’application Tipiṭaka",
                "Lecture d’un sutta, pali et traduction côte à côte",
                "Recherche dans le Canon par mot ou par sens",
                "Graphe de concepts reliant les passages proches",
                "Prise de notes sur un passage",
            ],
        }),
        instagram: Some(InstagramBlock {
            h: "Sur Instagram",
            tagline: "Un sutta par jour, avec une courte réflexion.",
            cta: "Suivre sur Instagram",
        }),
        sections: &[
            Section {
                h: "Ce que ça coûte",
                p: &[],
                list: &[
                    "Les serveurs qui font tourner la recherche et son index par le sens — hébergement dans le cloud.",
                    "Le nom de domaine et le réseau devant le site.",
                    "L’abonnement développeur Apple qui maintient l’app sur l’App Store.",
                    "Le calcul nécessaire pour construire et reconstruire les index et les traductions.",
                    "Le temps : construire, corriger, ajouter des langues.",
                ],
                table: None,
                cta: None,
            },
            Section {
                h: "Comment aider",
                p: &[
                    "Un don sur Ko-fi, ponctuel ou mensuel, va directement vers les coûts ci-dessus.",
                ],
                list: &[],
                table: None,
                cta: Some(Cta {
                    label: "Soutenir sur Ko-fi",
                    href: "https://ko-fi.com/tipitaka",
                }),
            },
            Section {
                h: "D’autres façons d’aider",
                p: &[
                    "Le Dhamma a toujours été offert librement ; ceci permet de continuer à l’offrir.",
                ],
                list: &[
                    "Partager un sutta que vous avez trouvé.",
                    "Noter l’app sur l’[App Store](https://apps.apple.com/app/apple-store/id6760995639?pt=128678815&ct=PaliWebSupport&mt=8).",
                    "Signaler une erreur : écrire à riviere.thibault@icloud.com.",
                ],
                table: None,
                cta: None,
            },
        ],
    },
    updated: "Dernière mise à jour : 2026-10-01.",
};
