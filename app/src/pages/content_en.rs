//! The footer pages in English: generated from today's site's
//! `web/src/locales/en.json` `pages.*` (verbatim; `{{publisher}}`,
//! `{{contact}}` and `{{date}}` resolved; the Legal page and the two
//! pointers to it dropped — task 001, fix round 4; AI mentions removed —
//! task 004). Data, not code: regenerate rather than edit by hand.

use super::{AppBlock, Cta, Href, InstagramBlock, PageText, Pages, Section, Table, ToolBlock};

/// Every footer page, in this language.
pub(super) const PAGES: Pages = Pages {
    acknowledgements: PageText {
        title: "Acknowledgements",
        description: "Credits for the suttas, translations and dictionary that PaliCanon searches, and their licences.",
        intro: &[
            "PaliCanon is a search engine, not a source. Everything it lets you find was made by others and given away; this page says by whom, and under what licence.",
        ],
        tools: &[],
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "The suttas",
                p: &[
                    "The English translations are by **Bhikkhu Sujato**, who translated the four main Nikāyas and much of the Khuddaka and dedicated the whole to the public domain. The Pāli text is the **Mahāsaṅgīti Tipiṭaka** edition prepared by the Dhamma Society. Both are published by [SuttaCentral](https://suttacentral.net) under the Creative Commons CC0 dedication, together with the sutta titles, blurbs and reference scheme that PaliCanon follows. Every result links back to SuttaCentral; please read the full suttas there, and consider [supporting them](https://suttacentral.net/donations).",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "French and Vietnamese translations",
                p: &[
                    "When PaliCanon is used in French or Vietnamese, sutta titles come from the translations published by [SuttaCentral](https://suttacentral.net) (each translation remains its translator's), wherever a sutta has one; the search reads them too, so a question asked in French or Vietnamese is answered from those texts. The French texts (1,184 suttas) are by **Môhan Wijayaratna**, **Rémy Zins**, **Claude & Chandhana Le Ninan**, **Christelle Ah-Mouck**, **Marion Ah-Mouck**, **Fernand Hû**, **U Ko Lay**, **U Tin U (Myaung)**, **Florent Robert**, **Indavati**, **Régis Xhardé**, and the teams of **buddha-vacana.org** and **Canonpali.org**. The Vietnamese texts (4,240 suttas) are by **Thích Minh Châu**, **Bhikkhu Indacanda** and **Phan Tuấn Anh**. All were collected from [SuttaCentral](https://suttacentral.net) and are reproduced unchanged; they belong to their translators.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "The dictionary",
                p: &[
                    "Pāli word meanings come from the **Digital Pāḷi Dictionary** by Bhikkhu Bodhirasa, [digitalpalidictionary.github.io](https://digitalpalidictionary.github.io), licensed [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/). PaliCanon is non-commercial and reproduces the entries unchanged; any further use of the dictionary data must respect that licence.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "With gratitude",
                p: &[
                    "To Bhikkhu Sujato, for translating the Sutta Piṭaka and giving it to everyone; to Bhikkhu Bodhirasa, for the dictionary; to the SuttaCentral team, for publishing and maintaining the texts; and to the Dhamma Society's Mahāsaṅgīti project, for the Pāli edition.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    terms: PageText {
        title: "Terms of use",
        description: "The terms governing use of PaliCanon.",
        intro: &[
            "These terms govern your use of PaliCanon (the “Service”), published by Thibault Rivière (the “Publisher”). By using the Service you accept them.",
        ],
        tools: &[],
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "1. The Service",
                p: &[
                    "The Service lets you search the suttas of the Sutta Piṭaka by exact words and by meaning, read the passages found with links to their source, and look up Pāli words. It is provided free of charge and without registration.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "2. Acceptance",
                p: &[
                    "Using the Service means accepting these terms. You may stop using the Service at any time; data stored in your browser is yours to delete (see [Privacy](/privacy)).",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "3. Texts and intellectual property",
                p: &[
                    "The suttas are in the public domain and the dictionary is under CC BY-NC 4.0, as detailed in [Acknowledgements](/acknowledgements). The Service itself — its search engine, interface, indexes and name — is the property of the Publisher. You may use it for personal, non-commercial consultation. Automated bulk extraction of results, reproduction of the Service, or use of its API outside the public site requires the Publisher's prior written permission.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "4. Availability and liability",
                p: &[
                    "The Service is provided “as is”, free of charge. The Publisher does its best to keep it accurate and available but gives no warranty of any kind, and may change, suspend or discontinue it at any time. To the extent permitted by law, the Publisher is not liable for any loss arising from use of the Service, from unavailability, or from the content of third-party sites it links to. Nothing here limits liability that cannot be limited by law.",
                    "You agree not to disrupt the Service, to send abusive or automated traffic, or to attempt to reach systems other than the public site. The Publisher may restrict access from any source that does so.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "5. Governing law and changes",
                p: &[
                    "These terms are governed by the law of the Publisher's place of residence, without prejudice to mandatory consumer protections of your own country. The Publisher may update them; the date below is that of the current version, and continued use after a change constitutes acceptance of it.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    privacy: PageText {
        title: "Privacy",
        description: "PaliCanon's GDPR privacy notice: what data is processed, why, and for how long.",
        intro: &[
            "This is the privacy notice required by the General Data Protection Regulation (GDPR). The data controller is Thibault Rivière, contact riviere.thibault@icloud.com. PaliCanon has no accounts, sets no cookies, and uses no advertising or third-party tracking.",
        ],
        tools: &[],
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "1. What is processed, why, and on what basis",
                p: &[
                    "Please do not type personal information — yours or anyone else's — into the search box: queries are kept, and they are kept anonymous only because nothing identifying is attached to them.",
                ],
                list: &[],
                table: Some(Table {
                    head: &["Data", "Purpose", "Legal basis (GDPR art. 6)", "Kept"],
                    rows: &[
                        &[
                            "Search queries: the text typed, the country of origin, the time to the hour, and how the search went (number of results, correction applied, latency). No IP address and no identifier are attached.",
                            "Improving the search: seeing what readers look for, what finds nothing, which spellings and Pāli terms are used.",
                            "Legitimate interest (6.1.f): running and improving a free public service, with data that cannot be linked to a person.",
                            "Up to 24 months; aggregated statistics indefinitely.",
                        ],
                        &[
                            "Connection logs kept by the hosting providers: IP address, time, requested URL, user agent.",
                            "Security, abuse prevention, keeping the Service running.",
                            "Legitimate interest (6.1.f).",
                            "By the providers, under their own policies; typically 30 days or less.",
                        ],
                    ],
                }),
                cta: None,
            },
            Section {
                h: "2. Recipients and transfers",
                p: &[
                    "The site is served by Cloudflare, Inc. (USA) and the search API by Google Cloud (Google Ireland Limited; servers may be located in the EU or the USA). Both act as processors under their standard data processing terms, which include the European Commission's standard contractual clauses for transfers outside the EU. No data is sold or shared with anyone else.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "3. What stays in your browser",
                p: &[
                    "Your theme and language are stored by your browser (localStorage) and are never sent to the Service. This storage is strictly necessary to provide the features you asked for and therefore requires no consent banner. You can erase it at any time by clearing the site's data in your browser.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "4. Your rights",
                p: &[
                    "You have the rights of access, rectification, erasure, restriction, portability and objection provided by the GDPR, and the right to lodge a complaint with a supervisory authority (in France, the CNIL). Write to riviere.thibault@icloud.com. Note that search queries are kept without any identifier: the Service cannot find “your” queries, and therefore cannot return or delete them individually — which is also what keeps them anonymous.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    support: PageText {
        title: "Support",
        description: "What it costs to run PaliCanon and the Tipiṭaka app, and how you can help keep them free.",
        intro: &[
            "PaliCanon and the Tipiṭaka app for iPhone are free: no ads, no accounts, nothing about you for sale. One person builds them and keeps them running.",
        ],
        tools: &[
            ToolBlock {
                id: "search",
                h: "The search",
                tagline: "Search the whole Pāli Canon by the words you remember or by what you mean: in English, French and Vietnamese, with the passage and a link to read it.",
                cta: "Search the Canon",
                href: Href::Path("/"),
                shots: &[
                    "/support/search-home.jpg",
                    "/support/search-results.jpg",
                    "/support/search-words.jpg",
                    "/support/search-french.jpg",
                    "/support/search-dark.jpg",
                ],
                alts: &[
                    "The search's home",
                    "Results for a question",
                    "Results by exact words",
                    "A search in French",
                    "The dark theme",
                ],
            },
            ToolBlock {
                id: "learn",
                h: "Learn Pāli",
                tagline: "Learn the language of the suttas one word at a time, with words taken from the Canon itself and reviewed at a rhythm that follows your memory.",
                cta: "Start learning Pāli",
                href: Href::Url("https://learn.palicanon.app"),
                shots: &[
                    "/support/learn-1.jpg",
                    "/support/learn-2.jpg",
                    "/support/learn-3.jpg",
                    "/support/learn-4.jpg",
                    "/support/learn-5.jpg",
                ],
                alts: &[
                    "The home of Learn Pāli",
                    "Adding new terms to the deck",
                    "A card asking the meaning of a Pāli word",
                    "The answer, with a line from a sutta",
                    "A sutta line with a word to find",
                ],
            },
            ToolBlock {
                id: "map",
                h: "The map",
                tagline: "See the Pāli Canon as a map in three dimensions: suttas linked by their parallels and by what they say.",
                cta: "Open the map",
                href: Href::Url("https://map.palicanon.app"),
                shots: &[
                    "/support/map-parallels.jpg",
                    "/support/map-meaning.jpg",
                    "/support/map-cluster.jpg",
                    "/support/map-phone.jpg",
                ],
                alts: &[
                    "The parallels view, after SuttaCentral",
                    "The meaning view, by embeddings",
                    "A sutta and its closest neighbours",
                    "The collections and the similarity threshold",
                ],
            },
        ],
        app: Some(AppBlock {
            h: "The Tipiṭaka app",
            tagline: "The whole Pāli Canon on your iPhone, offline: Pāli and translations side by side, a dictionary on every word, study plans, notes — in English, French and Vietnamese.",
            cta: "Get the iPhone app",
            alts: &[
                "Home screen of the Tipiṭaka app",
                "Reading a sutta, Pāli and translation side by side",
                "Searching the Canon by word or by meaning",
                "Concept graph linking related passages",
                "Taking notes on a passage",
            ],
        }),
        instagram: Some(InstagramBlock {
            h: "On Instagram",
            tagline: "A sutta a day, with a short reflection.",
            cta: "Follow on Instagram",
        }),
        sections: &[
            Section {
                h: "What it costs",
                p: &[],
                list: &[
                    "The servers that run the search and its meaning-based index — cloud hosting.",
                    "The domain name and the network in front of the site.",
                    "The Apple developer membership that keeps the app on the App Store.",
                    "The computing to build and rebuild the indexes and the translations.",
                    "Time: building, fixing, adding languages.",
                ],
                table: None,
                cta: None,
            },
            Section {
                h: "How to help",
                p: &[
                    "A donation on Ko-fi, one-off or monthly, goes straight toward the costs above.",
                ],
                list: &[],
                table: None,
                cta: Some(Cta {
                    label: "Support on Ko-fi",
                    href: "https://ko-fi.com/tipitaka",
                }),
            },
            Section {
                h: "Other ways to help",
                p: &["The Dhamma has always been offered freely; this keeps the offering going."],
                list: &[
                    "Share a sutta you found with someone.",
                    "Rate the app on the [App Store](https://apps.apple.com/app/apple-store/id6760995639?pt=128678815&ct=PaliWebSupport&mt=8).",
                    "Report a mistake: write to riviere.thibault@icloud.com.",
                ],
                table: None,
                cta: None,
            },
        ],
    },
    updated: "Last updated 2026-10-01.",
};
